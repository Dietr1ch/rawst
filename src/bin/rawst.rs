use std::path::PathBuf;
use std::sync::atomic::Ordering;

use base64::{prelude::BASE64_STANDARD, Engine as Base64Engine};
use chrono::prelude::Local;
use itertools::EitherOrBoth::{Both, Left};
use itertools::Itertools;

use rawst::cli::args::get_command;
use rawst::cli::args::DownloadArgs;
use rawst::cli::args::RawstCommand;
use rawst::cli::args::ResumeArgs;
use rawst::core::config::FileDownloadConfig;
use rawst::core::engine::DownloadTaskArgs;
use rawst::core::engine::Engine;
use rawst::core::errors::RawstErr;
use rawst::core::history::HistoryManager;
use rawst::core::io::{config_exist, get_cache_sizes, read_links};

// List download task arguments
//
// NOTE: We should be able to do this in a more functional way.
//       The full Vec shouldn't be necessary.
async fn list_downloads(args: DownloadArgs) -> Result<Vec<DownloadTaskArgs>, RawstErr> {
    let mut downloads: Vec<DownloadTaskArgs> = Vec::new();

    for f_o in args.files.into_iter().zip_longest(args.output_file_path) {
        downloads.push(match f_o {
            Both(f, o) => DownloadTaskArgs {
                iri: f,
                output_path: Some(PathBuf::from(&o)),
            },
            Left(f) => DownloadTaskArgs {
                iri: f,
                output_path: None,
            },
            _ => {
                unreachable!();
            }
        });
    }
    if let Some(input_file) = args.input_file {
        let link_string = read_links(&input_file).await?;
        for url in link_string.split("\n") {
            downloads.push(DownloadTaskArgs {
                iri: url.to_string(),
                output_path: None,
            })
        }
    }

    Ok(downloads)
}

async fn download(args: DownloadArgs, config: FileDownloadConfig) -> Result<(), RawstErr> {
    // Check args downloads
    if args.files.is_empty() {
        // Nothing to download. While we are technically done, but this is
        // likely not the user intent.
        return Err(RawstErr::InvalidArgs);
    }
    if !args.output_file_path.is_empty() && args.files.len() != args.output_file_path.len() {
        // There's a mismatch of downloads and output file paths.
        // This is likely not the user intent.
        return Err(RawstErr::InvalidArgs);
    }

    // Initialise
    let mut engine = Engine::new(config.clone());
    let history_manager = HistoryManager::new(config.config_path.clone());

    // Start downloads
    for d in list_downloads(args).await? {
        let download_task = engine.create_http_task(d).await?;
        let current_time = Local::now();
        let encoded_timestamp_as_id =
            BASE64_STANDARD.encode(current_time.timestamp().to_be_bytes());

        history_manager.add_record(&download_task, &config, encoded_timestamp_as_id.clone())?;
        engine.http_download(download_task).await?;
        history_manager.update_record(encoded_timestamp_as_id)?;
    }

    Ok(())
}

async fn resume_download(args: ResumeArgs, config: FileDownloadConfig) -> Result<(), RawstErr> {
    let history_manager = HistoryManager::new(config.config_path.clone());

    for download_id in args.download_ids {
        match history_manager.get_record(&download_id)? {
            Some(data) => {
                // NOTE: I can also get total file size by getting content length through http_task object
                let (url, threads, file_name, status) = data;
                if status == "Pending" {
                    let mut download_config = config.clone();
                    download_config.threads = threads;

                    let (file_stem, _) = file_name.rsplit_once(".").unwrap();

                    let mut engine = Engine::new(config.clone());

                    let mut http_task = engine
                        .create_http_task(DownloadTaskArgs {
                            iri: url,
                            output_path: Some(PathBuf::from(&file_stem.trim())),
                        })
                        .await?;

                    let cache_sizes = get_cache_sizes(file_name, threads, config.clone()).unwrap();

                    http_task.calculate_x_offsets(&cache_sizes);

                    http_task
                        .total_downloaded
                        .fetch_add(cache_sizes.iter().sum::<u64>(), Ordering::SeqCst);

                    engine.http_download(http_task).await?
                } else {
                    println!("The file is already downloaded");
                    return Ok(());
                }
            }
            None => {
                println!("Record with id {:?} not found", download_id);
                return Ok(());
            }
        }

        history_manager.update_record(download_id)?;
    }

    Ok(())
}

// fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
//     generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
// }

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let command = get_command().ok_or(std::io::ErrorKind::InvalidInput)?;

    // TODO Support generating autocompletion files
    // if let Some(generator) = command.generator {
    //     eprintln!("Generating completion file for {generator:?}...");
    //     eprintln!("Generating completion file for {generator:?}...");
    //     print_completions(generator, &mut command);
    //     return ();
    // }

    let config = match config_exist() {
        true => FileDownloadConfig::load().await?,
        false => FileDownloadConfig::build().await?,
    };

    match command {
        RawstCommand::Download(download_args) => {
            println!("Download {:?}", download_args);
            download(download_args, config)
                .await
                .map_err(|_| std::io::ErrorKind::InvalidInput.into())
        }
        RawstCommand::Resume(resume_args) => {
            println!("Resume {:?}", resume_args);
            resume_download(resume_args, config)
                .await
                .map_err(|_| std::io::ErrorKind::InvalidInput.into())
        }
        RawstCommand::History(_history_args) => {
            let history_manager = HistoryManager::new(config.config_path);
            history_manager
                .get_history()
                .map_err(|_| std::io::ErrorKind::InvalidInput.into())
        }
    }
}

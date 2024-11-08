use clap::Parser;
use clap::{Args as ClapArgs, Subcommand};
use clap_complete::Shell;
use clap_num::number_range;

#[derive(Parser, Debug, PartialEq)]
#[command(name = "rawst", version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    pub command: Option<RawstCommand>,

    // Hack to default to `rawst download ...`
    // The setup to make Download the default subcommand come from,
    // - https://github.com/clap-rs/clap/discussions/4134#discussioncomment-3511528
    #[command(flatten)]
    pub downloads: Option<DownloadArgs>,

    // If provided, outputs the completion file for given shell
    #[arg(long = "generate", value_enum)]
    generator: Option<Shell>,
}

pub fn get_command() -> Option<RawstCommand> {
    let args = Args::parse();

    if let Some(download_args) = args.downloads {
        Some(RawstCommand::Download(download_args))
    } else {
        args.command
    }
}

#[derive(Subcommand, Debug, PartialEq)]
pub enum RawstCommand {
    /// Download files
    Download(DownloadArgs),
    /// Resume partial downloads
    Resume(ResumeArgs),
    /// View download history
    History(HistoryArgs),
}

#[derive(ClapArgs, Debug, PartialEq)]
pub struct DownloadArgs {
    // Configuration
    /// Maximum amount of threads used to download
    ///
    /// Limited to 8 threads to avoid throttling
    #[arg(
      short, long,
      default_value="8",
      value_parser=limit_max_download_threads
  )]
    pub threads: u8,

    // Arguments
    // Inputs
    /// File where to look for download IRIs
    #[arg(short, long, default_value=None)]
    pub input_file: Option<String>,

    /// The IRIs to download
    #[arg(default_value = "")]
    pub files: Vec<String>,

    // Outputs
    /// The path to the downloaded files
    #[arg(long, default_value = "")]
    pub output_file_path: Vec<String>,
}

#[derive(ClapArgs, Debug, PartialEq)]
pub struct ResumeArgs {
    /// The Downloads to resume
    ///
    /// TODO: Default to resume the last download
    #[arg(default_value = "")]
    pub download_ids: Vec<String>,
}

#[derive(ClapArgs, Debug, PartialEq)]
pub struct HistoryArgs {}

fn limit_max_download_threads(s: &str) -> Result<u8, String> {
    const MAX_DOWNLOAD_THREADS: u8 = 8;
    number_range(s, 0, MAX_DOWNLOAD_THREADS)
}

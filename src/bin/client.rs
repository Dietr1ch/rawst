pub mod rawst {
    tonic::include_proto!("rawst");
}
use rawst::rawst_client::RawstClient;
use rawst::InfoRequest;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    // Server address
    #[clap(short, long, default_value = "http://[::1]:50051")]
    server_address: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut client = RawstClient::connect(args.server_address).await?;

    let request = tonic::Request::new(InfoRequest {});

    let response = client.info(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}

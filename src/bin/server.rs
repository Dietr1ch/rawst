pub mod rawst {
    use tonic::{Request, Response, Status};

    tonic::include_proto!("rawst"); // Proto package name
    use rawst_server::Rawst;

    #[derive(Default, Debug)]
    pub struct RawstImpl {}

    impl RawstImpl {
        pub fn new() -> Self {
            Self {}
        }
    }

    #[tonic::async_trait]
    impl Rawst for RawstImpl {
        async fn info(
            &self,
            request: Request<InfoRequest>,
        ) -> Result<Response<InfoResponse>, Status> {
            println!("Got a request: {:?}", request);

            Ok(Response::new(InfoResponse {
                version: env!("CARGO_PKG_VERSION").to_string(),
            }))
        }
    }
}

use std::net::SocketAddr;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Server address
    #[clap(short, long, default_value = "[::1]:50051")]
    address: SocketAddr,
}

use rawst::rawst_server::RawstServer;
use rawst::RawstImpl;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    console_subscriber::init();

    println!("Starting up server at '{:?}'", &args.address);
    let rawst_service = RawstImpl::new();

    Server::builder()
        .add_service(RawstServer::new(rawst_service))
        .serve(args.address)
        .await?;

    Ok(())
}

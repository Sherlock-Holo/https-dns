use anyhow::Result;
use clap::Parser;

use crate::cli::Args;
use crate::local::UdpListener;
use crate::upstream::HttpsClient;

pub mod bootstrap;
pub mod cache;
pub mod cli;
pub mod local;
pub mod upstream;
pub mod utils;

pub async fn run() -> Result<()> {
    tracing_subscriber::fmt().init();

    let args = Args::parse();

    let https_client = HttpsClient::new(&args.upstream, args.bootstrap_upstream.as_deref()).await?;
    let udp_listener = UdpListener::new(args.local_address, https_client).await?;

    udp_listener.listen().await;

    Ok(())
}

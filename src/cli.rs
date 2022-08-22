use std::net::SocketAddr;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(version, about)]
pub struct Args {
    #[clap(short, long)]
    pub local_address: SocketAddr,

    #[clap(short, long)]
    pub upstream: String,

    #[clap(short, long)]
    pub bootstrap_upstream: Option<String>,
}

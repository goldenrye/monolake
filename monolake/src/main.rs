#![feature(impl_trait_in_assoc_type)]

use anyhow::Result;
use clap::Parser;

use monolake_core::{config::Config, print_logo};
use runtimes::Runtimes;
use servers::Servers;
mod runtimes;
mod servers;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path of the config file
    #[clap(short, long, value_parser)]
    config: String,
}

#[monoio::main(timer_enabled = true)]
async fn main() -> Result<()> {
    print_logo();

    let args = Args::parse();
    let config = Config::load(args.config.to_owned()).await?;
    let (io_config, servers) = (config.runtime, config.servers);
    let mut servers = Servers::from(servers);
    servers.start(Runtimes::new(io_config)).await
}

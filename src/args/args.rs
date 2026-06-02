use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "hoatzin")]
#[command(version = "0.1.0")]
#[command(about = "Executes asynchronous workflows", long_about = None)]
pub struct Args {
    #[arg(short, long, value_name = "DIR")]
    pub workflows: PathBuf,
}

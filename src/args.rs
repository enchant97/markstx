use clap::{Parser, Subcommand};
use clio::{Input, Output};

#[derive(Debug, Subcommand)]
pub enum Commands {
    Compile {
        #[clap(long, short, value_parser)]
        input: Input,
        #[clap(long, short, value_parser, default_value = "-")]
        output: Output,
    },
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

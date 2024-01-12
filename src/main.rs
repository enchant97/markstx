mod args;
mod frontmatter;
mod functions;
mod processor;

use args::Args;
use clap::Parser;
use rust_embed::RustEmbed;

use crate::processor::Processor;

#[derive(RustEmbed)]
#[folder = "templates/"]
struct TemplateAssets;

fn main() {
    let args = Args::parse();

    let document = Processor::render_document(args.file);

    println!("{}", document);
}

mod args;
mod frontmatter;
mod functions;
mod processor;
mod utils;

use args::Args;
use clap::Parser;
use rust_embed::RustEmbed;

use crate::{processor::Processor, utils::DEFAULT_DOC_EXT};

#[derive(RustEmbed)]
#[folder = "templates/"]
struct TemplateAssets;

fn main() {
    let args = Args::parse();

    let mut root_doc_path = args.file;
    if root_doc_path.extension().is_none() {
        root_doc_path.set_extension(DEFAULT_DOC_EXT);
    }

    let document = Processor::render_document(root_doc_path);

    println!("{}", document);
}

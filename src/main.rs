mod args;
mod functions;
mod utils;

use std::sync::{Arc, RwLock};

use args::Args;
use clap::Parser;
use rust_embed::RustEmbed;

use crate::utils::DEFAULT_DOC_EXT;
use markstx_core::{
    frontmatter::{split_by_frontmatter, FrontMatter},
    processor::Processor,
};

#[derive(RustEmbed)]
#[folder = "templates/"]
struct TemplateAssets;

fn main() {
    let args = Args::parse();

    let mut root_doc_path = args.file;
    if root_doc_path.extension().is_none() {
        root_doc_path.set_extension(DEFAULT_DOC_EXT);
    }

    let source_content = std::fs::read_to_string(&root_doc_path).unwrap();
    let (frontmatter_raw, content_raw) = split_by_frontmatter(&source_content);
    let frontmatter = match frontmatter_raw {
        Some(frontmatter_raw) => FrontMatter::from_yaml_str(&frontmatter_raw).unwrap(),
        None => FrontMatter::default(),
    };

    let processor = Arc::new(RwLock::new(Processor::new(frontmatter)));

    {
        let mut p = processor.write().unwrap();
        p.set_template_loader(|name| {
            let actual_name = match name.ends_with(".tmpl") {
                true => name.to_owned(),
                false => format!("{name}.html.tmpl"),
            };
            if let Some(asset) = TemplateAssets::get(&actual_name) {
                return Ok(std::str::from_utf8(&asset.data).ok().map(|v| v.to_owned()));
            }
            Ok(None)
        });

        p.add_function("", "include", functions::make_include(processor.clone()));
        p.add_function("", "execute_command", functions::execute_command);
        p.add_function("", "convert_csv", functions::convert_csv);
        p.add_function("", "lorem_ipsum", functions::lorem_ipsum);
    }

    let p = processor.read().unwrap();
    let document = p.render_content_str(root_doc_path, &content_raw).unwrap();

    println!("{}", document);
}

mod args;
mod functions;
mod utils;

use anyhow::{Context, Result};
use args::Args;
use clap::Parser;
use rust_embed::RustEmbed;
use std::{
    io::{Read, Write},
    process::exit,
    sync::{Arc, RwLock},
};

use markstx_core::{
    frontmatter::{split_by_frontmatter, FrontMatter},
    processor::Processor,
};

#[derive(RustEmbed)]
#[folder = "templates/"]
struct TemplateAssets;

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        args::Commands::Compile {
            mut input,
            mut output,
        } => {
            if input.is_std() {
                eprintln!("input cannot be stdin, it must be a file");
                exit(1);
            }
            let mut source_content = String::new();
            input
                .read_to_string(&mut source_content)
                .with_context(|| format!("failed to read from {}", input.path()))?;

            let (frontmatter_raw, content_raw) = split_by_frontmatter(&source_content);
            let frontmatter = match frontmatter_raw {
                Some(frontmatter_raw) => FrontMatter::from_yaml_str(&frontmatter_raw)
                    .context("failed to process document frontmatter")?,
                None => FrontMatter::default(),
            };

            let processor = Arc::new(RwLock::new(Processor::new(frontmatter)));

            {
                let mut p = processor.write().expect("failed to gain lock on processor");
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

            let p = processor.read().expect("failed to gain lock on processor");
            let document = p
                .render_content_str(input.path().path(), &content_raw)
                .context("failed to process document")?;

            output
                .write_all(document.as_bytes())
                .with_context(|| format!("failed to read from {}", input.path()))?;
        }
    }
    Ok(())
}

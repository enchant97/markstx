mod args;
mod frontmatter;
mod functions;

use std::sync::{Arc, RwLock};

use args::Args;
use clap::Parser;
use minijinja::{context, Environment};
use pulldown_cmark::Options;
use rust_embed::RustEmbed;

use crate::frontmatter::{split_by_frontmatter, FrontMatter};

#[derive(RustEmbed)]
#[folder = "templates/"]
struct TemplateAssets;

fn setup_template_env(shared_env: Arc<RwLock<Environment<'static>>>) {
    let env = shared_env.clone();
    let mut env = env.write().unwrap();
    env.set_loader(|name| {
        let actual_name = match name.ends_with(".tmpl") {
            true => name.to_owned(),
            false => format!("{name}.html.tmpl"),
        };
        if let Some(asset) = TemplateAssets::get(&actual_name) {
            return Ok(std::str::from_utf8(&asset.data).ok().map(|v| v.to_owned()));
        }
        Ok(None)
    });
    env.add_function("include", functions::make_include(shared_env.clone()));
    env.add_function("execute_command", functions::execute_command);
    env.add_function("convert_csv", functions::convert_csv);
    env.add_function("lorem_ipsum", functions::lorem_ipsum);
}

fn main() {
    let args = Args::parse();

    let template_env = Arc::new(RwLock::new(Environment::new()));

    setup_template_env(template_env.clone());

    let source_content = std::fs::read_to_string(args.file.to_str().unwrap()).unwrap();

    let (frontmatter_raw, content_raw) = split_by_frontmatter(&source_content);

    let frontmatter = match frontmatter_raw {
        Some(frontmatter_raw) => FrontMatter::from_yaml_str(&frontmatter_raw).unwrap(),
        None => FrontMatter::default(),
    };

    let abs_path = args.file.canonicalize().unwrap();
    let current_dir = abs_path.parent().unwrap().to_str().unwrap();

    let enriched_md = template_env
        .read()
        .unwrap()
        .render_str(
            &content_raw,
            context! {
                frontmatter,
                current_dir,
            },
        )
        .unwrap();

    let md_parser = pulldown_cmark::Parser::new_ext(&enriched_md, Options::all());
    let mut content = String::new();
    pulldown_cmark::html::push_html(&mut content, md_parser);

    println!("{}", content);
}

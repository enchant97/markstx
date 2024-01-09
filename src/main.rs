mod args;
mod frontmatter;
mod functions;

use args::Args;
use clap::Parser;
use minijinja::{context, Environment};
use pulldown_cmark::Options;
use rust_embed::RustEmbed;

use crate::frontmatter::{split_by_frontmatter, FrontMatter};

#[derive(RustEmbed)]
#[folder = "templates/"]
struct TemplateAssets;

fn create_template_env() -> Environment<'static> {
    let mut env = Environment::new();
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
    env.add_function("execute_command", functions::execute_command);
    env.add_function("convert_csv", functions::convert_csv);
    env.add_function("lorem_ipsum", functions::lorem_ipsum);
    env
}

fn main() {
    let args = Args::parse();

    let template_env = create_template_env();

    let source_content = std::fs::read_to_string(args.file.to_str().unwrap()).unwrap();

    let (frontmatter_raw, content_raw) = split_by_frontmatter(&source_content);

    let frontmatter = match frontmatter_raw {
        Some(frontmatter_raw) => FrontMatter::from_yaml_str(&frontmatter_raw).unwrap(),
        None => FrontMatter::default(),
    };

    let base_tmpl = template_env
        .get_template("_internal/base")
        .expect("internal template not found");

    let enriched_md = template_env
        .render_str(
            &content_raw,
            context! {
                frontmatter,
            },
        )
        .unwrap();

    let md_parser = pulldown_cmark::Parser::new_ext(&enriched_md, Options::all());
    let mut content_body = String::new();
    pulldown_cmark::html::push_html(&mut content_body, md_parser);

    let final_output = base_tmpl
        .render(context! {
            frontmatter,
            content_body,
        })
        .unwrap();

    println!("{}", final_output);
}

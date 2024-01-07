mod args;
mod functions;

use args::Args;
use clap::Parser;
use minijinja::Environment;
use pulldown_cmark::Options;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "templates/"]
struct TemplateAssets;

fn create_template_env() -> Environment<'static> {
    let mut env = Environment::new();
    env.set_loader(|name| {
        if let Some(asset) = TemplateAssets::get(&format!("{name}.html.tmpl")) {
            return Ok(std::str::from_utf8(&asset.data).ok().map(|v| v.to_owned()));
        }
        Ok(None)
    });
    env.add_function("csv_to_table", functions::csv_to_table);
    env.add_function("lorem_ipsum", functions::lorem_ipsum);
    env
}

fn main() {
    let args = Args::parse();

    let template_env = create_template_env();

    let source_content = std::fs::read_to_string(args.file.to_str().unwrap()).unwrap();

    let enriched_md = template_env.render_str(&source_content, {}).unwrap();

    let md_parser = pulldown_cmark::Parser::new_ext(&enriched_md, Options::all());
    let mut output = String::new();
    pulldown_cmark::html::push_html(&mut output, md_parser);

    println!("{}", output);
}

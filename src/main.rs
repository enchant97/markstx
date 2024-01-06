mod args;
mod functions;

use args::Args;
use clap::Parser;
use pulldown_cmark::Options;
use rust_embed::RustEmbed;
use tera::Tera;

#[derive(RustEmbed)]
#[folder = "templates/"]
struct TemplateAsset;

fn main() {
    let args = Args::parse();

    let templates = TemplateAsset::iter().map(|filename| {
        (
            filename.to_string(),
            TemplateAsset::get(&filename).expect(&format!("failed to load asset '{}'", filename)),
        )
    });
    let mut tera = Tera::default();
    for template in templates {
        tera.add_raw_template(
            &template.0,
            std::str::from_utf8(&template.1.data).expect("failed to convert file into utf8"),
        )
        .unwrap();
    }
    tera.register_function("csv_to_table", functions::make_csv_to_table());
    tera.add_template_file(&args.file, None).unwrap();

    let enriched_md = tera
        .render(args.file.to_str().unwrap(), &tera::Context::new())
        .unwrap();

    let md_parser = pulldown_cmark::Parser::new_ext(&enriched_md, Options::all());
    let mut output = String::new();
    pulldown_cmark::html::push_html(&mut output, md_parser);

    println!("{}", output);
}

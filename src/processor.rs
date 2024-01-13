use std::{
    fs::read_to_string,
    path::Path,
    sync::{Arc, RwLock},
};

use minijinja::{context, Environment, Value};
use pulldown_cmark::Options;

use crate::{
    frontmatter::{split_by_frontmatter, FrontMatter},
    functions, TemplateAssets,
};

pub struct Processor {
    template_env: Arc<RwLock<Environment<'static>>>,
    pub frontmatter: FrontMatter,
}

impl Processor {
    fn new(frontmatter: FrontMatter) -> Self {
        let template_env = Environment::new();
        Self {
            template_env: Arc::new(RwLock::new(template_env)),
            frontmatter,
        }
    }
    pub(crate) fn setup(processor: Arc<RwLock<Self>>) {
        let p = processor.clone();
        let p = p.read().unwrap();
        let mut env = p.template_env.write().unwrap();

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

        env.add_function("_include", functions::make_include(processor));
        env.add_function("_execute_command", functions::execute_command);
        env.add_function("_convert_csv", functions::convert_csv);
        env.add_function("_lorem_ipsum", functions::lorem_ipsum);
    }
    pub fn create_context<P: AsRef<Path>>(&self, filepath: P) -> Value {
        let abs_path = filepath.as_ref().canonicalize().unwrap();
        let current_dir = abs_path.parent().unwrap().to_str().unwrap();
        context! {
            frontmatter => self.frontmatter,
            current_dir,
        }
    }
    pub fn render_content<P: AsRef<Path>>(&self, path: P) -> String {
        let content = std::fs::read_to_string(&path).unwrap();
        let enriched_md = self
            .template_env
            .read()
            .unwrap()
            .render_str(&content, self.create_context(path))
            .unwrap();

        let parser = pulldown_cmark::Parser::new_ext(&enriched_md, Options::all());
        let mut content = String::new();
        pulldown_cmark::html::push_html(&mut content, parser);

        content
    }
    pub fn render_content_str<P: AsRef<Path>>(&self, path: P, content: &str) -> String {
        let enriched_md = self
            .template_env
            .read()
            .unwrap()
            .render_str(&content, self.create_context(path))
            .unwrap();

        let parser = pulldown_cmark::Parser::new_ext(&enriched_md, Options::all());
        let mut content = String::new();
        pulldown_cmark::html::push_html(&mut content, parser);

        content
    }
    pub fn render_document<P: AsRef<Path>>(path: P) -> String {
        let source_content = read_to_string(&path).unwrap();
        let (frontmatter_raw, content_raw) = split_by_frontmatter(&source_content);

        let frontmatter = match frontmatter_raw {
            Some(frontmatter_raw) => FrontMatter::from_yaml_str(&frontmatter_raw).unwrap(),
            None => FrontMatter::default(),
        };

        let processor = Arc::new(RwLock::new(Self::new(frontmatter)));

        Self::setup(processor.clone());

        let processor = processor.read().unwrap();
        processor.render_content_str(path, &content_raw)
    }
}

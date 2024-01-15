use core::fmt;
use std::path::Path;

use minijinja::{
    functions::Function,
    value::{FunctionArgs, FunctionResult},
    Environment,
};
use pulldown_cmark::Options;

use crate::{context::Context, frontmatter::FrontMatter};

#[derive(Debug)]
pub enum ProcessorErrorType {
    PathInvalid(String),
    Parser(minijinja::Error),
    IO(std::io::Error),
    InvalidFrontmatter,
    Generic(String),
}

#[derive(Debug)]
pub struct ProcessorError {
    pub error: ProcessorErrorType,
}

impl ProcessorError {
    pub fn new_generic(detail: &str) -> Self {
        Self {
            error: ProcessorErrorType::Generic(detail.to_owned()),
        }
    }
}

impl std::error::Error for ProcessorError {}

impl fmt::Display for ProcessorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.error {
            ProcessorErrorType::PathInvalid(v) => write!(f, "path '{v}' invalid"),
            ProcessorErrorType::Parser(e) => write!(f, "{e}"),
            ProcessorErrorType::IO(e) => write!(f, "{e}"),
            ProcessorErrorType::InvalidFrontmatter => write!(f, "frontmatter invalid"),
            ProcessorErrorType::Generic(v) => write!(f, "{v}"),
        }
    }
}

impl From<ProcessorErrorType> for ProcessorError {
    fn from(v: ProcessorErrorType) -> Self {
        Self { error: v }
    }
}

impl From<minijinja::Error> for ProcessorError {
    fn from(v: minijinja::Error) -> Self {
        Self {
            error: ProcessorErrorType::Parser(v),
        }
    }
}

impl From<ProcessorError> for minijinja::Error {
    fn from(v: ProcessorError) -> Self {
        match v.error {
            ProcessorErrorType::PathInvalid(msg) => {
                Self::new(minijinja::ErrorKind::InvalidOperation, msg)
            }
            ProcessorErrorType::Parser(e) => e,
            ProcessorErrorType::IO(e) => {
                Self::new(minijinja::ErrorKind::InvalidOperation, e.to_string())
            }
            ProcessorErrorType::InvalidFrontmatter => Self::new(
                minijinja::ErrorKind::InvalidOperation,
                "invalid document frontmatter".to_owned(),
            ),
            ProcessorErrorType::Generic(v) => Self::new(minijinja::ErrorKind::InvalidOperation, v),
        }
    }
}

pub struct Processor {
    template_env: Environment<'static>,
    pub frontmatter: FrontMatter,
}

impl Processor {
    pub fn new(frontmatter: FrontMatter) -> Self {
        let template_env = Environment::new();
        Self {
            template_env,
            frontmatter,
        }
    }
    pub fn set_template_loader<F>(&mut self, f: F)
    where
        F: Fn(&str) -> Result<Option<String>, minijinja::Error> + Send + Sync + 'static,
    {
        self.template_env.set_loader(f);
    }
    pub fn add_function<F, Rv, Args>(&mut self, plugin_name: &str, name: &str, f: F)
    where
        F: Function<Rv, Args> + for<'a> Function<Rv, <Args as FunctionArgs<'a>>::Output>,
        Rv: FunctionResult,
        Args: for<'a> FunctionArgs<'a>,
    {
        self.template_env
            .add_function(format!("{plugin_name}_{name}"), f);
    }
    pub fn create_context<P: AsRef<Path>>(&self, filepath: P) -> Result<Context, ProcessorError> {
        let abs_path = filepath
            .as_ref()
            .canonicalize()
            .map_err(|e| ProcessorErrorType::PathInvalid(e.to_string()))?;
        let current_dir = abs_path.parent().ok_or(ProcessorErrorType::PathInvalid(
            "given path has no parent".to_string(),
        ))?;

        Ok(Context {
            frontmatter: self.frontmatter.clone(),
            current_dir: current_dir.to_path_buf(),
        })
    }
    /// Renders the given string, using path as the current file-path being rendered.
    pub fn render_content_str<P: AsRef<Path>>(
        &self,
        path: P,
        content: &str,
    ) -> Result<String, ProcessorError> {
        let enriched_md = self
            .template_env
            .render_str(content, self.create_context(path)?)?;

        let parser = pulldown_cmark::Parser::new_ext(&enriched_md, Options::all());
        let mut content = String::new();
        pulldown_cmark::html::push_html(&mut content, parser);

        Ok(content)
    }
    /// Same as `render_content_str`, but instead loads directly from given file-path.
    pub fn render_content<P: AsRef<Path>>(&self, path: P) -> Result<String, ProcessorError> {
        let content = std::fs::read_to_string(&path).map_err(ProcessorErrorType::IO)?;
        self.render_content_str(path, &content)
    }
}

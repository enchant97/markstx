use crate::utils::DEFAULT_DOC_EXT;
use markstx_core::processor::{Processor, ProcessorError};
use minijinja::{functions::Function, value::Kwargs, Error, Value};
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    process::{Command, Stdio},
    sync::{Arc, RwLock},
};

type FnResult<Rt> = Result<Rt, Error>;

pub fn make_include(
    processor: Arc<RwLock<Processor>>,
) -> impl Function<FnResult<String>, (String, String)> {
    move |dir: String, filename: String| -> FnResult<String> {
        let mut path = PathBuf::from(dir).join(filename);
        if path.extension().is_none() {
            path.set_extension(DEFAULT_DOC_EXT);
        }
        let content = processor
            .read()
            .expect("failed to gain lock on processor")
            .render_content(path)
            .map_err(Into::<Error>::into)?;
        Ok(content)
    }
}

pub fn execute_command(command: String, options: Kwargs) -> Result<String, Error> {
    match options.get::<Option<Vec<String>>>("args")? {
        Some(args) => Ok(std::str::from_utf8(
            &Command::new(command)
                .args(&args)
                .stdout(Stdio::piped())
                .spawn()
                .map_err(|_| ProcessorError::new_generic("failed to execute process"))?
                .wait_with_output()
                .map_err(|_| ProcessorError::new_generic("failed to wait on process"))?
                .stdout,
        )
        .map_err(|_| ProcessorError::new_generic("executed process gave back non utf8 characters"))?
        .to_owned()),
        None => Err(Error::new(
            minijinja::ErrorKind::MissingArgument,
            "missing 'args'",
        )),
    }
}

pub fn lorem_ipsum(options: Kwargs) -> Result<String, Error> {
    match options.get("words")? {
        Some(words) => Ok(lipsum::lipsum(words)),
        None => Err(Error::new(
            minijinja::ErrorKind::MissingArgument,
            "missing 'words'",
        )),
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ParsedCSV {
    pub headers: Vec<String>,
    pub records: Vec<Vec<String>>,
}

fn pass_csv<R>(reader: &mut csv::Reader<R>, has_headers: bool) -> Result<ParsedCSV, Error>
where
    R: std::io::Read,
{
    let headers = match has_headers {
        false => vec![],
        true => reader
            .headers()
            .map_err(|e| Error::new(minijinja::ErrorKind::InvalidOperation, e.to_string()))?
            .into_iter()
            .map(|v| v.to_owned())
            .collect::<Vec<String>>(),
    };
    let mut records = Vec::new();
    for result in reader.records() {
        let columns = result
            .map_err(|e| Error::new(minijinja::ErrorKind::InvalidOperation, e.to_string()))?
            .into_iter()
            .map(|v| v.to_owned())
            .collect::<Vec<String>>();
        records.push(columns);
    }
    Ok(ParsedCSV { headers, records })
}

pub fn convert_csv(options: Kwargs) -> Result<Value, Error> {
    let has_headers: bool = options
        .get::<Option<bool>>("has_headers")
        .map_err(|e| Error::new(minijinja::ErrorKind::InvalidOperation, e.to_string()))?
        .unwrap_or_default();

    let mut reader_builder = csv::ReaderBuilder::new();
    let reader = reader_builder.trim(csv::Trim::All).has_headers(has_headers);

    if let Some(path) = options.get::<Option<String>>("path")? {
        let mut reader = reader
            .from_path(path)
            .map_err(|e| Error::new(minijinja::ErrorKind::InvalidOperation, e.to_string()))?;
        let parsed = pass_csv(&mut reader, has_headers)?;
        Ok(Value::from_serializable(&parsed))
    } else if let Some(content) = options.get::<Option<String>>("content")? {
        let mut reader = csv::Reader::from_reader(content.as_bytes());
        let parsed = pass_csv(&mut reader, has_headers)?;
        Ok(Value::from_serializable(&parsed))
    } else {
        Err(Error::new(
            minijinja::ErrorKind::MissingArgument,
            "missing 'path' or 'content'",
        ))
    }
}

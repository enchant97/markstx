use minijinja::{context, functions::Function, value::Kwargs, Environment, Error, Value};
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    process::{Command, Stdio},
    sync::{Arc, RwLock},
};

type FnResult<Rt> = Result<Rt, Error>;

pub fn make_include(
    env: Arc<RwLock<Environment<'static>>>,
) -> impl Function<FnResult<String>, (String, String)> {
    move |dir: String, filename: String| -> FnResult<String> {
        let full_path = PathBuf::from(dir).join(filename);
        let current_dir = &full_path.parent().unwrap().to_str().unwrap();
        let content = std::fs::read_to_string(&full_path).unwrap();
        env.read()
            .unwrap()
            .render_str(
                &content,
                context! {
                    current_dir,
                },
            )
            .map_err(|e| {
                minijinja::Error::new(minijinja::ErrorKind::InvalidOperation, e.to_string())
            })
    }
}

pub fn execute_command(command: String, options: Kwargs) -> Result<String, Error> {
    match options.get::<Option<Vec<String>>>("args")? {
        Some(args) => Ok(std::str::from_utf8(
            &Command::new(command)
                .args(&args)
                .stdout(Stdio::piped())
                .spawn()
                .unwrap()
                .wait_with_output()
                .unwrap()
                .stdout,
        )
        .unwrap()
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

fn pass_csv<R>(reader: &mut csv::Reader<R>) -> Result<ParsedCSV, Error>
where
    R: std::io::Read,
{
    let headers = reader
        .headers()
        .map_err(|e| Error::new(minijinja::ErrorKind::InvalidOperation, e.to_string()))?
        .into_iter()
        .map(|v| v.to_owned())
        .collect::<Vec<String>>();
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
    if let Some(path) = options.get::<Option<String>>("path")? {
        let mut reader = csv::Reader::from_path(&path)
            .map_err(|e| Error::new(minijinja::ErrorKind::InvalidOperation, e.to_string()))?;
        let parsed = pass_csv(&mut reader)?;
        Ok(Value::from_serializable(&parsed))
    } else if let Some(content) = options.get::<Option<String>>("content")? {
        let mut reader = csv::Reader::from_reader(content.as_bytes());
        let parsed = pass_csv(&mut reader)?;
        Ok(Value::from_serializable(&parsed))
    } else {
        Err(Error::new(
            minijinja::ErrorKind::MissingArgument,
            "missing 'path' or 'content'",
        ))
    }
}

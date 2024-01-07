use minijinja::{value::Kwargs, Error};
use std::{
    fs,
    process::{Command, Stdio},
};

pub fn execute_command(command: String, options: Kwargs) -> Result<String, Error> {
    match options.get::<Option<Vec<String>>>("args")? {
        Some(args) => Ok(std::str::from_utf8(
            &Command::new(&command)
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

pub fn csv_to_table(options: Kwargs) -> Result<String, Error> {
    match options.get("path")? {
        Some(path) => {
            let content = fs::read_to_string::<&str>(path)
                .map_err(|e| Error::new(minijinja::ErrorKind::InvalidOperation, e.to_string()))?;
            let mut lines = content.lines();
            let mut output = String::new();

            output.push_str("\n<table>");

            if let Some(header) = &lines.next() {
                let columns: Vec<&str> = header.split(',').collect();
                output.push_str("<thead>");
                output.push_str(
                    &columns
                        .into_iter()
                        .map(|v| format!("<th>{}</th>", v.trim()))
                        .collect::<Vec<_>>()
                        .join(""),
                );
                output.push_str("</thead>");
            }

            for row in lines.into_iter() {
                let columns: Vec<&str> = row.split(',').collect();
                output.push_str("<tr>");
                output.push_str(
                    &columns
                        .into_iter()
                        .map(|v| format!("<td>{}</td>", v.trim()))
                        .collect::<Vec<_>>()
                        .join(""),
                );
                output.push_str("</tr>");
            }

            output.push_str("</table>\n");

            Ok(output.into())
        }
        None => Err(Error::new(
            minijinja::ErrorKind::MissingArgument,
            "missing 'path'",
        )),
    }
}

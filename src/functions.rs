use std::{collections::HashMap, fs};
use tera::{from_value, Function, Result, Value};

pub fn make_csv_to_table() -> impl Function {
    Box::new(move |args: &HashMap<String, Value>| -> Result<Value> {
        match args.get("path") {
            Some(val) => match from_value::<String>(val.clone()) {
                Ok(path) => {
                    let content = fs::read_to_string(path)?;
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
                Err(_) => Err("oops".into()),
            },
            None => Err("missing 'path'".into()),
        }
    })
}

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct FrontMatter {}

impl FrontMatter {
    pub fn from_yaml_str(content: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(content)
    }
}

pub fn split_by_frontmatter(content: &str) -> (Option<String>, String) {
    let lines: Vec<&str> = content.lines().collect();
    let mut final_content = content.to_owned();
    let mut frontmatter = None;

    if lines.len() > 2 && lines[0] == "---" {
        for (i, line) in lines[1..].into_iter().enumerate() {
            if *line == "---" {
                frontmatter = Some(lines[1..i + 1].join("\n"));
                final_content = lines[i + 2..].join("\n");
                break;
            }
        }
    }

    (frontmatter, final_content)
}

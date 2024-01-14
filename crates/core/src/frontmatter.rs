use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum PageOrientation {
    #[serde(rename = "portrait")]
    Portrait,
    #[serde(rename = "landscape")]
    Landscape,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum PageSize {
    Absolute {
        name: String,
        orientation: PageOrientation,
    },
    Values {
        width: String,
        height: String,
    },
}

impl Default for PageSize {
    fn default() -> Self {
        Self::Absolute {
            name: String::from("A4"),
            orientation: PageOrientation::Portrait,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PageMargin {
    pub top: String,
    pub right: String,
    pub bottom: String,
    pub left: String,
}

impl Default for PageMargin {
    fn default() -> Self {
        const DEFAULT_MARGIN: &str = "0.75in";
        Self {
            top: String::from(DEFAULT_MARGIN),
            right: String::from(DEFAULT_MARGIN),
            bottom: String::from(DEFAULT_MARGIN),
            left: String::from(DEFAULT_MARGIN),
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct PageConfig {
    #[serde(default)]
    pub size: PageSize,
    #[serde(default)]
    pub margin: PageMargin,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DocConfig {
    #[serde(default)]
    pub title: String,
}

impl Default for DocConfig {
    fn default() -> Self {
        Self {
            title: String::from("Untitled Document"),
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct FrontMatter {
    #[serde(default)]
    pub doc: DocConfig,
    #[serde(default)]
    pub page: PageConfig,
}

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
        for (i, line) in lines[1..].iter().enumerate() {
            if *line == "---" {
                frontmatter = Some(lines[1..i + 1].join("\n"));
                final_content = lines[i + 2..].join("\n");
                break;
            }
        }
    }

    (frontmatter, final_content)
}

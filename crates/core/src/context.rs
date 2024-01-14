use std::path::PathBuf;

use crate::frontmatter::FrontMatter;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Context {
    pub frontmatter: FrontMatter,
    pub current_dir: PathBuf,
}

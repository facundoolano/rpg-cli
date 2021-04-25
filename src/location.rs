use serde::{Deserialize, Serialize};
use std::path;

#[derive(Serialize, Deserialize, Debug)]
pub struct Location {
    pub path: path::PathBuf,
}

impl Location {
    pub fn from(path: &str) -> Self {
        // FIXME force ~ into home
        let path = path::Path::new(path).canonicalize().unwrap();
        Self { path }
    }

    pub fn home() -> Self {
        Self {
            path: dirs::home_dir().unwrap(),
        }
    }

    pub fn is_home(&self) -> bool {
        self.path == dirs::home_dir().unwrap()
    }

    // TODO add a unit test for this
    /// Move this location one step towards the given destination
    pub fn walk_towards(&mut self, dest: &Self) {
        if !dest.path.starts_with(&self.path) {
            self.path = self.path.parent().unwrap().to_path_buf();
        } else {
            let next = dest
                .path
                .strip_prefix(&self.path)
                .unwrap()
                .components()
                .next()
                .unwrap();
            self.path = self.path.join(next);
        }
    }
}

impl PartialEq for Location {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path.to_string_lossy())
    }
}

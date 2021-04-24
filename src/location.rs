use serde::{Deserialize, Serialize};
use std::path;

#[derive(Serialize, Deserialize, Debug)]
pub struct Location {
    pub path: path::PathBuf,
}

fn comp_to_string(component: path::Component) -> String {
    // FIXME is there a better way for this?
    component.as_os_str().to_string_lossy().to_string()
}

impl Location {
    pub fn from(path: &str) -> Self {
        // TODO sanitize path
        Self {
            path: path::Path::new(path).to_path_buf(),
        }
    }

    pub fn home() -> Self {
        Self {
            path: dirs::home_dir().unwrap(),
        }
    }

    pub fn is_home(&self) -> bool {
        self.path == dirs::home_dir().unwrap()
    }

    // FIXME docstring
    // FIXME tests
    pub fn walk_to(&self, dest: &Self) -> Vec<String> {
        let common: String = self
            .path
            .components()
            .zip(dest.path.components())
            .take_while(|(c1, c2)| c1 == c2)
            .map(|(c1, _c2)| comp_to_string(c1))
            .collect::<Vec<String>>()
            .join("/");

        let dest_steps = dest.path.strip_prefix(&common).unwrap().components();
        let current_steps = self.path.strip_prefix(common).unwrap().components().rev();

        // FIXME this probably is missing the common path
        current_steps
            .chain(dest_steps)
            .map(comp_to_string)
            .collect()
    }
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path.to_string_lossy())
    }
}

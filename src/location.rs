use serde::{Deserialize, Serialize};
use std::path;

#[derive(Serialize, Deserialize, Debug)]
pub struct Location {
    pub path: path::PathBuf,
}

impl Location {
    /// Build a location from the given path string.
    /// The path is validated to exist and converted to it's canonical form.
    pub fn from(path: &str) -> Self {
        // if input doesn't come from shell, we want to interpret ~ as home ourselves
        let path = if path.starts_with('~') {
            // TODO figure out these string lossy stuff
            let home_str = dirs::home_dir().unwrap().to_string_lossy().to_string();
            path.replacen("~", &home_str, 1)
        } else {
            path.to_string()
        };

        // TODO gracefully return the error when path is not found
        let path = path::Path::new(&path).canonicalize().unwrap();
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

    /// Move this location one step towards the given destination
    pub fn walk_towards(&mut self, dest: &Self) {
        if !dest.path.starts_with(&self.path) {
            self.path = self.path.parent().unwrap().to_path_buf();
        } else if dest != self {
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

    pub fn distance_from(&self, other: &Self) -> i32 {
        let mut current = self.path.as_path();
        let dest = other.path.as_path();

        let mut distance = 0;
        while !dest.starts_with(&current) {
            current = current.parent().unwrap();
            distance += 1;
        }
        let dest = dest.strip_prefix(current).unwrap();
        distance + dest.components().count() as i32
    }

    pub fn distance_from_home(&self) -> i32 {
        self.distance_from(&Location::home())
    }
}

impl PartialEq for Location {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let home = dirs::home_dir().unwrap().to_string_lossy().to_string();
        let mut loc = self.path.to_string_lossy().replace(&home, "~");
        if loc == "~" {
            loc = "home".to_string();
        }
        write!(f, "{}", loc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from() {
        assert_ne!(Location::from("/"), Location::home());
        assert_eq!(Location::from("~"), Location::from("~/"));
        assert_eq!(Location::from("~/."), Location::from("~/"));
        // FIXME this only works if /usr/bin exists
        assert_eq!(Location::from("/usr"), Location::from("/usr/bin/../"));
    }

    #[test]
    fn test_walk_towards() {
        let mut source = location_from("/Users/facundo/dev/");
        let dest = location_from("/");

        source.walk_towards(&dest);
        assert_eq!(location_from("/Users/facundo/"), source);
        source.walk_towards(&dest);
        assert_eq!(location_from("/Users/"), source);
        source.walk_towards(&dest);
        assert_eq!(location_from("/"), source);
        source.walk_towards(&dest);
        assert_eq!(location_from("/"), source);

        let mut source = location_from("/Users/facundo/rust/rpg");
        let dest = location_from("/Users/facundo/erlang/app");

        source.walk_towards(&dest);
        assert_eq!(location_from("/Users/facundo/rust/"), source);
        source.walk_towards(&dest);
        assert_eq!(location_from("/Users/facundo/"), source);
        source.walk_towards(&dest);
        assert_eq!(location_from("/Users/facundo/erlang"), source);
        source.walk_towards(&dest);
        assert_eq!(location_from("/Users/facundo/erlang/app"), source);
    }

    #[test]
    fn test_distance() {
        let distance = |from, to| location_from(from).distance_from(&location_from(to));

        assert_eq!(distance("/Users/facundo", "/Users/facundo"), 0);
        assert_eq!(distance("/Users/facundo", "/Users/facundo/other"), 1);
        assert_eq!(distance("/Users/facundo/other", "/Users/facundo/"), 1);
        assert_eq!(distance("/Users/facundo/other", "/"), 3);
        assert_eq!(distance("/", "/Users/facundo/other"), 3);
        assert_eq!(distance("/Users/rusty/cage", "/Users/facundo/other"), 4);
        assert_eq!(distance("/Users/facundo/other", "/Users/rusty/cage"), 4);
        assert_eq!(Location::home().distance_from_home(), 0);
    }

    /// test-only equivalent for Location::from, specifically to bypass
    /// path existence checks.
    fn location_from(path: &str) -> Location {
        let path = path::Path::new(path);
        Location {
            path: path.to_path_buf(),
        }
    }
}

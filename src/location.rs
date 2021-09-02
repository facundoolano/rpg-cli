use serde::{Deserialize, Serialize};
use std::path;

#[derive(Serialize, Deserialize, Debug, Eq, Clone)]
pub struct Location {
    path: path::PathBuf,
}

impl Location {
    /// Build a location from the given path string.
    /// The path is validated to exist and converted to it's canonical form.
    pub fn from(path: &str) -> Result<Self, std::io::Error> {
        // if input doesn't come from shell, we want to interpret ~ as home ourselves
        let mut path = patch_oldpwd(path);
        if path.starts_with('~') {
            // TODO figure out these string lossy stuff
            let home_str = dirs::home_dir().unwrap().to_string_lossy().to_string();
            path = path.replacen("~", &home_str, 1)
        }

        let path = path::Path::new(&path);
        // this is a replacement to std::fs::canonicalize()
        // that circumvents windows quirks with paths
        let path = dunce::canonicalize(&path)?;
        Ok(Self { path })
    }

    pub fn path_string(&self) -> String {
        self.path.to_string_lossy().to_string()
    }

    pub fn home() -> Self {
        Self {
            path: dirs::home_dir().unwrap(),
        }
    }

    pub fn is_home(&self) -> bool {
        self.path == dirs::home_dir().unwrap()
    }

    pub fn is_rpg_dir(&self) -> bool {
        self.path == dirs::home_dir().unwrap().join(".rpg")
    }

    /// Return a new location that it's one dir closer to the given destination.
    pub fn go_to(&self, dest: &Self) -> Self {
        let next = if dest.path.starts_with(&self.path) {
            let self_len = self.path.components().count();
            dest.path.components().take(self_len + 1).collect()
        } else {
            self.path.parent().unwrap().to_path_buf()
        };
        Self { path: next }
    }

    fn distance_from(&self, other: &Self) -> Distance {
        let mut current = self.path.as_path();
        let dest = other.path.as_path();

        let mut distance = 0;
        while !dest.starts_with(&current) {
            current = current.parent().unwrap();
            distance += 1;
        }
        let dest = dest.strip_prefix(current).unwrap();
        let len = distance + dest.components().count() as i32;
        Distance::from(len)
    }

    pub fn distance_from_home(&self) -> Distance {
        self.distance_from(&Location::home())
    }
}

/// To match the `cd` behavior, when the path '-' is passed try to
/// go to the previous location based on $OLDPWD.
/// If that env var is missing go home.
fn patch_oldpwd(path: &str) -> String {
    if path == "-" {
        if let Ok(val) = std::env::var("OLDPWD") {
            val
        } else {
            String::from("~")
        }
    } else {
        path.to_string()
    }
}

impl PartialEq for Location {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl std::hash::Hash for Location {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.path.hash(state)
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

/// Some decisions are made branching on whether the distance from the home dir
/// is small, medium or large. This enum encapsulate the definition of those.
pub enum Distance {
    Near(i32),
    Mid(i32),
    Far(i32),
}

impl Distance {
    pub fn from(len: i32) -> Self {
        match len {
            n if n <= 6 => Self::Near(len),
            n if n <= 15 => Self::Mid(len),
            _ => Self::Far(len),
        }
    }

    pub fn len(&self) -> i32 {
        match self {
            Distance::Near(s) => *s,
            Distance::Mid(s) => *s,
            Distance::Far(s) => *s,
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_from() {
        assert_ne!(Location::from("/").unwrap(), Location::home());
        assert_eq!(Location::from("~").unwrap(), Location::from("~/").unwrap());
        assert_eq!(
            Location::from("~/.").unwrap(),
            Location::from("~/").unwrap()
        );
        // FIXME this only works if /usr/bin exists
        // assert_eq!(
        //     Location::from("/usr").unwrap(),
        //     Location::from("/usr/bin/../").unwrap()
        // );
    }

    #[test]
    fn test_walk_towards() {
        let source = location_from("/Users/facundo/dev/");
        let dest = location_from("/");

        let source = source.go_to(&dest);
        assert_eq!(location_from("/Users/facundo/"), source);
        let source = source.go_to(&dest);
        assert_eq!(location_from("/Users/"), source);
        let source = source.go_to(&dest);
        assert_eq!(location_from("/"), source);
        let source = source.go_to(&dest);
        assert_eq!(location_from("/"), source);

        let source = location_from("/Users/facundo/rust/rpg");
        let dest = location_from("/Users/facundo/erlang/app");

        let source = source.go_to(&dest);
        assert_eq!(location_from("/Users/facundo/rust/"), source);
        let source = source.go_to(&dest);
        assert_eq!(location_from("/Users/facundo/"), source);
        let source = source.go_to(&dest);
        assert_eq!(location_from("/Users/facundo/erlang"), source);
        let source = source.go_to(&dest);
        assert_eq!(location_from("/Users/facundo/erlang/app"), source);
    }

    #[test]
    fn test_distance() {
        let distance = |from, to| location_from(from).distance_from(&location_from(to));

        assert_eq!(distance("/Users/facundo", "/Users/facundo").len(), 0);
        assert_eq!(distance("/Users/facundo", "/Users/facundo/other").len(), 1);
        assert_eq!(distance("/Users/facundo/other", "/Users/facundo/").len(), 1);
        assert_eq!(distance("/Users/facundo/other", "/").len(), 3);
        assert_eq!(distance("/", "/Users/facundo/other").len(), 3);
        assert_eq!(
            distance("/Users/rusty/cage", "/Users/facundo/other").len(),
            4
        );
        assert_eq!(
            distance("/Users/facundo/other", "/Users/rusty/cage").len(),
            4
        );
        assert_eq!(Location::home().distance_from_home().len(), 0);
    }

    /// test-only equivalent for Location::from, specifically to bypass
    /// path existence checks.
    pub fn location_from(path: &str) -> Location {
        let path = path::Path::new(path);
        Location {
            path: path.to_path_buf(),
        }
    }
}

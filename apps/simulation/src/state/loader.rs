
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct SimStateFile {
    pub metadata: Metadata,
    pub world: WorldSection,
    pub spawn: SpawnSection,
}

#[derive(Debug, Deserialize)]
pub struct Metadata {
    pub version: String,
    pub description: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct WorldSection {
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct Rectangle {
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
}

impl Rectangle {
    pub fn is_valid(&self) -> bool {
        self.min_x < self.max_x && self.min_y < self.max_y
    }

    pub fn width(&self) -> f32 {
        self.max_x - self.min_x
    }

    pub fn height(&self) -> f32 {
        self.max_y - self.min_y
    }
}

#[derive(Debug, Deserialize)]
pub struct SpawnSection {
    pub count: usize,
    pub behavior: String,
    pub spawn_zone: Rectangle,
    pub target_zone: Rectangle,
}

#[derive(Debug)]
pub enum StateLoadError {
    FileNotFound(String),
    ParseError(String),
    IoError(String),
}

impl std::fmt::Display for StateLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StateLoadError::FileNotFound(path) => write!(f, "State file not found: {}", path),
            StateLoadError::ParseError(msg) => write!(f, "Failed to parse state file: {}", msg),
            StateLoadError::IoError(msg) => write!(f, "IO error reading state file: {}", msg),
        }
    }
}

impl std::error::Error for StateLoadError {}

impl SimStateFile {
    pub fn load_from_file(path: &Path) -> Result<Self, StateLoadError> {
        let path_str = path.display().to_string();

        if !path.exists() {
            return Err(StateLoadError::FileNotFound(path_str));
        }

        let content = fs::read_to_string(path)
            .map_err(|e| StateLoadError::IoError(format!("{}: {}", path_str, e)))?;

        let state: SimStateFile = toml::from_str(&content)
            .map_err(|e| StateLoadError::ParseError(format!("{}: {}", path_str, e)))?;

        Ok(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::PathBuf;

    #[test]
    fn test_load_valid_state_file() {
        let content = r#"
[metadata]
version = "1.0"
description = "Test state"
created_at = "2025-11-04T12:00:00Z"

[world]
width = 200.0
height = 150.0

[spawn]
count = 10
behavior = "seeking"
spawn_zone = { min_x = 0.0, max_x = 100.0, min_y = 0.0, max_y = 100.0 }
target_zone = { min_x = 200.0, max_x = 300.0, min_y = 200.0, max_y = 300.0 }
"#;

        let mut temp_file = std::env::temp_dir();
        temp_file.push("test_state.toml");

        {
            let mut file = fs::File::create(&temp_file).unwrap();
            file.write_all(content.as_bytes()).unwrap();
        }

        let result = SimStateFile::load_from_file(&temp_file);
        assert!(result.is_ok());

        let state = result.unwrap();
        assert_eq!(state.metadata.version, "1.0");
        assert_eq!(state.world.width, 200.0);
        assert_eq!(state.spawn.count, 10);
        assert_eq!(state.spawn.behavior, "seeking");
        assert!(state.spawn.spawn_zone.is_valid());
        assert!(state.spawn.target_zone.is_valid());

        fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_load_minimal_state_file() {
        let content = r#"
[metadata]
version = "1.0"
description = "Minimal test state"
created_at = "2025-11-04T12:00:00Z"

[world]
width = 180.0
height = 130.0

[spawn]
count = 5
behavior = "seeking"
spawn_zone = { min_x = -50.0, max_x = 50.0, min_y = -50.0, max_y = 50.0 }
target_zone = { min_x = 100.0, max_x = 150.0, min_y = 100.0, max_y = 150.0 }
"#;

        let mut temp_file = std::env::temp_dir();
        temp_file.push("test_minimal_state.toml");

        {
            let mut file = fs::File::create(&temp_file).unwrap();
            file.write_all(content.as_bytes()).unwrap();
        }

        let result = SimStateFile::load_from_file(&temp_file);
        assert!(result.is_ok());

        let state = result.unwrap();
        assert_eq!(state.metadata.version, "1.0");
        assert_eq!(state.world.width, 180.0);
        assert_eq!(state.spawn.count, 5);
        assert_eq!(state.spawn.behavior, "seeking");

        fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_load_nonexistent_file() {
        let path = PathBuf::from("/nonexistent/path/state.toml");
        let result = SimStateFile::load_from_file(&path);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            StateLoadError::FileNotFound(_)
        ));
    }

    #[test]
    fn test_load_invalid_toml() {
        let content = "this is not valid TOML content [[[";
        let mut temp_file = std::env::temp_dir();
        temp_file.push("test_invalid.toml");

        {
            let mut file = fs::File::create(&temp_file).unwrap();
            file.write_all(content.as_bytes()).unwrap();
        }

        let result = SimStateFile::load_from_file(&temp_file);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), StateLoadError::ParseError(_)));

        fs::remove_file(temp_file).ok();
    }
}

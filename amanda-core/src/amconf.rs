//! # Amconf Format
//! 
//! Configuration format for Amanda OS tools.
//! Supports profiles, credentials, and tool-specific settings.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::Result;

/// Configuration profile (for multi-account/tool setups)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Profile {
    pub name: String,
    pub settings: HashMap<String, serde_json::Value>,
}

/// Encrypted credential entry (stub for now)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Credential {
    pub name: String,
    pub credential_type: String,
    /// Encrypted value (base64)
    pub encrypted_value: String,
}

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Config {
    /// Version of the config format
    pub version: String,
    
    /// Default profile name
    pub default_profile: Option<String>,
    
    /// Available profiles
    pub profiles: HashMap<String, Profile>,
    
    /// Stored credentials (encrypted)
    pub credentials: Vec<Credential>,
    
    /// Global settings
    pub globals: HashMap<String, serde_json::Value>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            default_profile: None,
            profiles: HashMap::new(),
            credentials: Vec::new(),
            globals: HashMap::new(),
        }
    }
}

impl Config {
    /// Create a new empty configuration
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Add a profile
    pub fn add_profile(&mut self, profile: Profile) -> &mut Self {
        if self.default_profile.is_none() {
            self.default_profile = Some(profile.name.clone());
        }
        self.profiles.insert(profile.name.clone(), profile);
        self
    }
    
    /// Get a profile by name
    pub fn get_profile(&self, name: &str) -> Option<&Profile> {
        self.profiles.get(name)
    }
    
    /// Get the default profile
    pub fn default_profile(&self) -> Option<&Profile> {
        self.default_profile
            .as_ref()
            .and_then(|name| self.profiles.get(name))
    }
    
    /// Set a global setting
    pub fn set_global(&mut self, key: impl Into<String>, value: impl Serialize) -> Result<&mut Self> {
        self.globals.insert(key.into(), serde_json::to_value(value)?);
        Ok(self)
    }
    
    /// Get a global setting
    pub fn get_global<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        self.globals.get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }
    
    /// Serialize to JSON
    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }
    
    /// Deserialize from JSON
    pub fn from_json(json: &str) -> Result<Self> {
        Ok(serde_json::from_str(json)?)
    }
    
    /// Load from file
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let json = std::fs::read_to_string(path)?;
        Self::from_json(&json)
    }
    
    /// Save to file (creates parent directories if needed)
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = self.to_json()?;
        std::fs::write(path, json)?;
        Ok(())
    }
    
    /// Load or create default config
    pub fn load_or_default(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if path.exists() {
            Self::load(path)
        } else {
            Ok(Self::new())
        }
    }
}

/// Config directory utilities (XDG Base Directory compliant)
pub mod dirs {
    use std::env::var_os;
    use std::path::PathBuf;
    
    /// Get the base config directory (~/.config or $XDG_CONFIG_HOME)
    fn base_config_dir() -> Option<PathBuf> {
        if let Some(xdg_config) = var_os("XDG_CONFIG_HOME") {
            Some(PathBuf::from(xdg_config))
        } else if let Some(home) = var_os("HOME") {
            Some(PathBuf::from(home).join(".config"))
        } else {
            None
        }
    }
    
    /// Get the config directory for Amanda OS tools
    pub fn config_dir() -> Option<PathBuf> {
        base_config_dir().map(|d| d.join("amanda"))
    }
    
    /// Get the default config file path
    pub fn default_config_path() -> Option<PathBuf> {
        config_dir().map(|d| d.join("config.json"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_builder() {
        let mut config = Config::new();
        
        config.set_global("timeout", 30).unwrap();
        config.add_profile(Profile {
            name: "default".to_string(),
            settings: HashMap::new(),
        });
        
        assert_eq!(config.default_profile, Some("default".to_string()));
        assert_eq!(config.get_global::<i32>("timeout"), Some(30));
    }
    
    #[test]
    fn test_serde_roundtrip() {
        let mut config = Config::new();
        config.set_global("key", "value").unwrap();
        
        let json = config.to_json().unwrap();
        let restored = Config::from_json(&json).unwrap();
        
        assert_eq!(config.globals, restored.globals);
    }
}

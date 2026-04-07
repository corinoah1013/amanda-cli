//! # Amanda Core
//! 
//! Core library for the Amanda OS CLI ecosystem.
//! Provides shared types, formats, and utilities for:
//! - `.amaudit` — Immutable audit trail format with hash chains
//! - `.amrpt` — Structured report format
//! - `.amconf` — Configuration format

pub mod amaudit;
pub mod amrpt;
pub mod amconf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Common metadata for all Amanda OS formats
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AmandaMetadata {
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub generator: String,
    pub hostname: Option<String>,
}

impl AmandaMetadata {
    pub fn new(generator: impl Into<String>) -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            created_at: Utc::now(),
            generator: generator.into(),
            hostname: Some(gethostname::gethostname()),
        }
    }
}

/// Get system hostname
pub mod gethostname {
    pub fn gethostname() -> String {
        std::env::var("HOSTNAME")
            .or_else(|_| std::fs::read_to_string("/etc/hostname").map(|s| s.trim().to_string()))
            .unwrap_or_else(|_| "unknown".to_string())
    }
}

/// Format bytes to human-readable string
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];

    if bytes == 0 {
        return "0 B".to_string();
    }

    let exp = (bytes as f64).log(1024.0).min(UNITS.len() as f64 - 1.0) as usize;
    let value = bytes as f64 / 1024f64.powi(exp as i32);

    if exp == 0 {
        format!("{} {}", bytes, UNITS[0])
    } else {
        format!("{:.1} {}", value, UNITS[exp])
    }
}

/// Common error type for Amanda OS operations
#[derive(Debug, thiserror::Error)]
pub enum AmandaError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Hash chain verification failed at entry {index}")]
    HashChainViolation { index: usize },
    
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
    
    #[error("Configuration error: {0}")]
    Config(String),
}

pub type Result<T> = std::result::Result<T, AmandaError>;

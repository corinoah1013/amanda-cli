//! # Amaudit Format
//! 
//! Immutable audit trail format with hash chain (tamper-evident).
//! Each entry contains a SHA-256 hash of the previous entry,
//! creating a cryptographic chain that detects any modification.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{AmandaMetadata, Result};

/// Entry in an audit trail
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditEntry {
    /// Sequence number in the chain
    pub sequence: u64,
    
    /// Timestamp of the entry
    pub timestamp: DateTime<Utc>,
    
    /// Entry type/category
    pub entry_type: String,
    
    /// Source component/tool
    pub source: String,
    
    /// Actual payload (JSON value for flexibility)
    pub data: serde_json::Value,
    
    /// SHA-256 hash of the previous entry (empty for genesis)
    pub prev_hash: String,
    
    /// SHA-256 hash of this entry (calculated from all fields except this)
    pub hash: String,
}

impl AuditEntry {
    /// Create a new audit entry
    pub fn new(
        sequence: u64,
        entry_type: impl Into<String>,
        source: impl Into<String>,
        data: impl Serialize,
        prev_hash: impl Into<String>,
    ) -> Result<Self> {
        let data_json = serde_json::to_value(data)?;
        
        let mut entry = Self {
            sequence,
            timestamp: Utc::now(),
            entry_type: entry_type.into(),
            source: source.into(),
            data: data_json,
            prev_hash: prev_hash.into(),
            hash: String::new(), // Will be calculated
        };
        
        entry.hash = entry.calculate_hash();
        Ok(entry)
    }
    
    /// Calculate the hash of this entry (excluding the hash field itself)
    fn calculate_hash(&self) -> String {
        // Create a copy without the hash field for consistent hashing
        let hash_input = format!(
            "{}:{}:{}:{}:{}:{}",
            self.sequence,
            self.timestamp.to_rfc3339(),
            self.entry_type,
            self.source,
            self.data,
            self.prev_hash
        );
        
        let mut hasher = Sha256::new();
        hasher.update(hash_input.as_bytes());
        hex::encode(hasher.finalize())
    }
    
    /// Verify the integrity of this entry's hash
    pub fn verify_hash(&self) -> bool {
        self.hash == self.calculate_hash()
    }
}

/// Audit trail container
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditTrail {
    pub metadata: AmandaMetadata,
    pub entries: Vec<AuditEntry>,
}

impl AuditTrail {
    /// Create a new empty audit trail
    pub fn new(generator: impl Into<String>) -> Self {
        Self {
            metadata: AmandaMetadata::new(generator),
            entries: Vec::new(),
        }
    }
    
    /// Get the hash of the last entry (empty string if no entries)
    pub fn last_hash(&self) -> String {
        self.entries
            .last()
            .map(|e| e.hash.clone())
            .unwrap_or_default()
    }
    
    /// Add a new entry to the trail
    pub fn add_entry(
        &mut self,
        entry_type: impl Into<String>,
        source: impl Into<String>,
        data: impl Serialize,
    ) -> Result<&AuditEntry> {
        let sequence = self.entries.len() as u64;
        let prev_hash = self.last_hash();
        
        let entry = AuditEntry::new(sequence, entry_type, source, data, prev_hash)?;
        self.entries.push(entry);
        
        Ok(self.entries.last().unwrap())
    }
    
    /// Verify the entire hash chain
    /// Returns Ok(()) if valid, Err with the index of first violation
    pub fn verify_chain(&self) -> Result<()> {
        for (i, entry) in self.entries.iter().enumerate() {
            // Verify entry hash
            if !entry.verify_hash() {
                return Err(crate::AmandaError::HashChainViolation { index: i });
            }
            
            // Verify chain linkage (skip genesis)
            if i > 0 {
                let expected_prev_hash = &self.entries[i - 1].hash;
                if &entry.prev_hash != expected_prev_hash {
                    return Err(crate::AmandaError::HashChainViolation { index: i });
                }
            }
        }
        Ok(())
    }
    
    /// Serialize to JSON
    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }
    
    /// Deserialize from JSON
    pub fn from_json(json: &str) -> Result<Self> {
        Ok(serde_json::from_str(json)?)
    }
    
    /// Save to file
    pub fn save(&self, path: impl AsRef<std::path::Path>) -> Result<()> {
        let json = self.to_json()?;
        std::fs::write(path, json)?;
        Ok(())
    }
    
    /// Load from file
    pub fn load(path: impl AsRef<std::path::Path>) -> Result<Self> {
        let json = std::fs::read_to_string(path)?;
        Self::from_json(&json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_audit_entry_hash() {
        let entry = AuditEntry::new(0, "test", "test-source", "test data", "").unwrap();
        assert!(entry.verify_hash());
        assert!(!entry.hash.is_empty());
    }
    
    #[test]
    fn test_audit_chain() {
        let mut trail = AuditTrail::new("test");
        
        trail.add_entry("event1", "source", "data1").unwrap();
        trail.add_entry("event2", "source", "data2").unwrap();
        trail.add_entry("event3", "source", "data3").unwrap();
        
        assert_eq!(trail.entries.len(), 3);
        
        // Verify chain
        trail.verify_chain().unwrap();
        
        // Verify linkages
        assert_eq!(trail.entries[1].prev_hash, trail.entries[0].hash);
        assert_eq!(trail.entries[2].prev_hash, trail.entries[1].hash);
    }
    
    #[test]
    fn test_tamper_detection() {
        let mut trail = AuditTrail::new("test");
        trail.add_entry("event1", "source", "data1").unwrap();
        trail.add_entry("event2", "source", "data2").unwrap();
        
        // Tamper with first entry
        trail.entries[0].data = serde_json::json!("tampered");
        
        // Should fail verification
        assert!(trail.verify_chain().is_err());
    }
    
    #[test]
    fn test_serde_roundtrip() {
        let mut trail = AuditTrail::new("test");
        trail.add_entry("event1", "source", "data1").unwrap();
        
        let json = trail.to_json().unwrap();
        let restored = AuditTrail::from_json(&json).unwrap();
        
        assert_eq!(trail.entries.len(), restored.entries.len());
        assert_eq!(trail.entries[0].hash, restored.entries[0].hash);
    }
}

//! # Amrpt Format
//! 
//! Structured report format for Amanda OS.
//! Supports multiple output formats: JSON, text, CSV.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{AmandaMetadata, Result};

/// Report section with data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReportSection {
    /// Section title
    pub title: String,
    
    /// Section type (table, text, chart, etc.)
    pub section_type: SectionType,
    
    /// Section data (flexible JSON value)
    pub data: serde_json::Value,
    
    /// Optional metadata for this section
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SectionType {
    Table,
    Text,
    Metrics,
    Chart,
    List,
}

/// Structured report
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Report {
    pub metadata: ReportMetadata,
    pub sections: Vec<ReportSection>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReportMetadata {
    pub amanda: AmandaMetadata,
    pub report_type: String,
    pub title: String,
    pub description: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

impl Report {
    /// Create a new report
    pub fn new(
        generator: impl Into<String>,
        report_type: impl Into<String>,
        title: impl Into<String>,
    ) -> Self {
        Self {
            metadata: ReportMetadata {
                amanda: AmandaMetadata::new(generator),
                report_type: report_type.into(),
                title: title.into(),
                description: None,
                start_time: Some(Utc::now()),
                end_time: None,
            },
            sections: Vec::new(),
        }
    }
    
    /// Add a section to the report
    pub fn add_section(&mut self, section: ReportSection) -> &mut Self {
        self.sections.push(section);
        self
    }
    
    /// Add a table section
    pub fn add_table<T: Serialize>(
        &mut self,
        title: impl Into<String>,
        rows: Vec<T>,
    ) -> Result<&mut Self> {
        let data = serde_json::to_value(rows)?;
        self.add_section(ReportSection {
            title: title.into(),
            section_type: SectionType::Table,
            data,
            metadata: None,
        });
        Ok(self)
    }
    
    /// Add metrics section
    pub fn add_metrics(
        &mut self,
        title: impl Into<String>,
        metrics: HashMap<String, f64>,
    ) -> &mut Self {
        self.add_section(ReportSection {
            title: title.into(),
            section_type: SectionType::Metrics,
            data: serde_json::to_value(metrics).unwrap_or_default(),
            metadata: None,
        });
        self
    }
    
    /// Add text section
    pub fn add_text(&mut self, title: impl Into<String>, text: impl Into<String>) -> &mut Self {
        self.add_section(ReportSection {
            title: title.into(),
            section_type: SectionType::Text,
            data: serde_json::Value::String(text.into()),
            metadata: None,
        });
        self
    }
    
    /// Finalize the report (set end time)
    pub fn finalize(&mut self) -> &mut Self {
        self.metadata.end_time = Some(Utc::now());
        self
    }
    
    /// Serialize to JSON
    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }
    
    /// Serialize to compact JSON
    pub fn to_json_compact(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
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
    
    /// Render as plain text
    pub fn render_text(&self) -> String {
        let mut output = String::new();
        
        // Header
        output.push_str(&format!("{:=^60}\n", ""));
        output.push_str(&format!("  {}\n", self.metadata.title));
        output.push_str(&format!("  Type: {}\n", self.metadata.report_type));
        output.push_str(&format!("  Generated: {}\n", self.metadata.amanda.created_at));
        output.push_str(&format!("{:=^60}\n\n", ""));
        
        // Sections
        for section in &self.sections {
            output.push_str(&format!("## {}\n", section.title));

            match section.section_type {
                SectionType::Text => {
                    if let Some(text) = section.data.as_str() {
                        output.push_str(text);
                        output.push('\n');
                    }
                }
                SectionType::Metrics => {
                    if let Some(obj) = section.data.as_object() {
                        for (key, value) in obj {
                            output.push_str(&format!("  {}: {}\n", key, value));
                        }
                    }
                }
                SectionType::List => {
                    if let Some(arr) = section.data.as_array() {
                        for item in arr {
                            let text = item.as_str()
                                .map(|s| s.to_string())
                                .unwrap_or_else(|| item.to_string());
                            output.push_str(&format!("  - {}\n", text));
                        }
                    }
                }
                SectionType::Chart => {
                    if let Some(obj) = section.data.as_object() {
                        let max_val = obj.values()
                            .filter_map(|v| v.as_f64())
                            .fold(0.0_f64, f64::max);
                        for (key, value) in obj {
                            if let Some(v) = value.as_f64() {
                                let bar_width = 30usize;
                                let filled = if max_val > 0.0 {
                                    ((v / max_val) * bar_width as f64) as usize
                                } else {
                                    0
                                };
                                let bar = "█".repeat(filled) + &"░".repeat(bar_width - filled);
                                output.push_str(&format!("  {:20} │{}│ {:.1}\n", key, bar, v));
                            }
                        }
                    } else {
                        output.push_str(&serde_json::to_string_pretty(&section.data).unwrap_or_default());
                        output.push('\n');
                    }
                }
                SectionType::Table => {
                    output.push_str(&serde_json::to_string_pretty(&section.data).unwrap_or_default());
                    output.push('\n');
                }
            }

            output.push('\n');
        }
        
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[derive(Serialize)]
    struct TestRow {
        name: String,
        value: i32,
    }
    
    #[test]
    fn test_report_builder() {
        let mut report = Report::new("test", "system", "System Report");
        
        report.add_text("Summary", "This is a test report");
        
        let mut metrics = HashMap::new();
        metrics.insert("cpu".to_string(), 45.5);
        metrics.insert("memory".to_string(), 78.2);
        report.add_metrics("Resources", metrics);
        
        report.add_table("Data", vec![
            TestRow { name: "a".to_string(), value: 1 },
            TestRow { name: "b".to_string(), value: 2 },
        ]).unwrap();
        
        assert_eq!(report.sections.len(), 3);
        
        let text = report.render_text();
        assert!(text.contains("System Report"));
        assert!(text.contains("test report"));
        assert!(text.contains("cpu"));
    }
}

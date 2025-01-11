use anyhow::{Result, Context};
use serde::Serialize;
use std::path::Path;
use tokio::fs;
use tracing::info;

use crate::processing::Document;

#[derive(Debug, Serialize)]
pub struct Summary {
    pub original_path: String,
    pub summary: String,
    pub metadata: SummaryMetadata,
}

#[derive(Debug, Serialize)]
pub struct SummaryMetadata {
    pub original_size: u64,
    pub summary_size: u64,
    pub compression_ratio: f64,
}

pub trait OutputFormatter {
    fn format(&self, summaries: &[Summary]) -> Result<String>;
}

pub struct MarkdownFormatter;
impl OutputFormatter for MarkdownFormatter {
    fn format(&self, summaries: &[Summary]) -> Result<String> {
        let mut output = String::new();

        for summary in summaries {
            output.push_str(&format!("# Summary of {}\n\n", summary.original_path));
            output.push_str(&summary.summary);
            output.push_str("\n\n---\n\n");
            
            if summary.metadata.compression_ratio < 1.0 {
                output.push_str(&format!(
                    "_Compressed to {:.1}% of original size_\n\n",
                    summary.metadata.compression_ratio * 100.0
                ));
            }
        }

        Ok(output)
    }
}

pub struct JsonFormatter;
impl OutputFormatter for JsonFormatter {
    fn format(&self, summaries: &[Summary]) -> Result<String> {
        serde_json::to_string_pretty(summaries).context("Failed to serialize to JSON")
    }
}

pub struct PlainTextFormatter;
impl OutputFormatter for PlainTextFormatter {
    fn format(&self, summaries: &[Summary]) -> Result<String> {
        let mut output = String::new();

        for summary in summaries {
            output.push_str(&format!("=== {} ===\n\n", summary.original_path));
            output.push_str(&summary.summary);
            output.push_str("\n\n");
        }

        Ok(output)
    }
}

pub struct OutputWriter {
    formatter: Box<dyn OutputFormatter>,
}

impl OutputWriter {
    pub fn new(format: &str) -> Result<Self> {
        let formatter: Box<dyn OutputFormatter> = match format.to_lowercase().as_str() {
            "md" | "markdown" => Box::new(MarkdownFormatter),
            "json" => Box::new(JsonFormatter),
            "txt" | "text" => Box::new(PlainTextFormatter),
            _ => anyhow::bail!("Unsupported output format: {}", format),
        };

        Ok(Self { formatter })
    }

    pub async fn write(&self, summaries: Vec<Summary>, output_path: Option<&Path>) -> Result<()> {
        let formatted = self.formatter.format(&summaries)?;

        match output_path {
            Some(path) => {
                fs::write(path, formatted).await?;
                info!("Written output to {}", path.display());
            }
            None => println!("{}", formatted),
        }

        Ok(())
    }
}

impl Summary {
    pub fn new(document: &Document, summary: String) -> Self {
        let summary_size = summary.len() as u64;
        let compression_ratio = summary_size as f64 / document.metadata.file_size as f64;

        Self {
            original_path: document.path.to_string_lossy().into_owned(),
            summary,
            metadata: SummaryMetadata {
                original_size: document.metadata.file_size,
                summary_size,
                compression_ratio,
            },
        }
    }
} 
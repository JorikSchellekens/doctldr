use anyhow::Result;
use encoding_rs::Encoding;
use ignore::WalkBuilder;
use pulldown_cmark::{Parser, Event};
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::warn;
use regex::RegexBuilder;

#[derive(Debug)]
pub struct Document {
    pub path: PathBuf,
    pub content: String,
    pub format: DocumentFormat,
    pub metadata: DocumentMetadata,
}

#[derive(Debug)]
pub struct DocumentMetadata {
    pub file_size: u64,
    pub encoding: String,
    pub line_count: usize,
}

#[derive(Debug, PartialEq)]
pub enum DocumentFormat {
    Markdown,
    RestructuredText,
    Html,
    PlainText,
}

impl DocumentFormat {
    fn from_extension(path: &Path) -> Self {
        match path.extension().and_then(|e| e.to_str()) {
            Some("md") => Self::Markdown,
            Some("rst") => Self::RestructuredText,
            Some("html") | Some("htm") => Self::Html,
            _ => Self::PlainText,
        }
    }
}

pub struct DocumentProcessor {
    max_depth: usize,
    include_patterns: Vec<String>,
    exclude_patterns: Vec<String>,
}

impl DocumentProcessor {
    pub fn new(max_depth: usize, include_patterns: Vec<String>, exclude_patterns: Vec<String>) -> Self {
        Self {
            max_depth,
            include_patterns,
            exclude_patterns,
        }
    }

    pub async fn process_directory(&self, dir: &Path) -> Result<Vec<Document>> {
        let mut documents = Vec::new();
        let walker = WalkBuilder::new(dir)
            .max_depth(Some(self.max_depth))
            .standard_filters(true)
            .build();

        for entry in walker {
            let entry = entry?;
            let path = entry.path();

            if !self.should_process_file(path) {
                continue;
            }

            match self.process_file(path).await {
                Ok(doc) => documents.push(doc),
                Err(e) => warn!("Failed to process file {}: {}", path.display(), e),
            }
        }

        Ok(documents)
    }

    async fn process_file(&self, path: &Path) -> Result<Document> {
        let content = fs::read(path).await?;
        let (content, encoding) = detect_and_decode(&content);

        let metadata = DocumentMetadata {
            file_size: content.len() as u64,
            encoding: encoding.name().to_string(),
            line_count: content.lines().count(),
        };

        let format = DocumentFormat::from_extension(path);
        let content = self.preprocess_content(&content, &format)?;

        Ok(Document {
            path: path.to_owned(),
            content,
            format,
            metadata,
        })
    }

    fn should_process_file(&self, path: &Path) -> bool {
        if !path.is_file() {
            return false;
        }

        let path_str = path.to_string_lossy();
        
        // Check exclude patterns first
        if self.exclude_patterns.iter().any(|pattern| {
            let regex = RegexBuilder::new(&glob_to_regex(pattern))
                .case_insensitive(true)
                .build()
                .unwrap_or_else(|_| RegexBuilder::new(".^").build().unwrap());
            regex.is_match(&path_str)
        }) {
            return false;
        }

        // Then check include patterns
        self.include_patterns.iter().any(|pattern| {
            let regex = RegexBuilder::new(&glob_to_regex(pattern))
                .case_insensitive(true)
                .build()
                .unwrap_or_else(|_| RegexBuilder::new(".^").build().unwrap());
            regex.is_match(&path_str)
        })
    }

    fn preprocess_content(&self, content: &str, format: &DocumentFormat) -> Result<String> {
        match format {
            DocumentFormat::Markdown => self.process_markdown(content),
            DocumentFormat::Html => Ok(html2text::from_read(content.as_bytes(), 80)),
            DocumentFormat::RestructuredText => Ok(content.to_string()), // TODO: Implement RST processing
            DocumentFormat::PlainText => Ok(content.to_string()),
        }
    }

    fn process_markdown(&self, content: &str) -> Result<String> {
        let mut output = String::new();
        let parser = Parser::new(content);
        
        for event in parser {
            match event {
                Event::Text(text) => output.push_str(&text),
                Event::Code(code) => output.push_str(&code),
                Event::SoftBreak | Event::HardBreak => output.push('\n'),
                _ => {}
            }
        }

        Ok(output)
    }
}

fn detect_and_decode(content: &[u8]) -> (String, &'static Encoding) {
    // Try to detect BOM first
    if content.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return decode_with_encoding(content, encoding_rs::UTF_8);
    }
    if content.starts_with(&[0xFE, 0xFF]) || content.starts_with(&[0xFF, 0xFE]) {
        return decode_with_encoding(content, encoding_rs::UTF_16LE);
    }
    
    // Try UTF-8 first
    let (text, encoding, had_errors) = encoding_rs::UTF_8.decode(content);
    if !had_errors {
        return (text.into_owned(), encoding_rs::UTF_8);
    }
    
    // If UTF-8 fails, try other common encodings
    for encoding in &[
        encoding_rs::WINDOWS_1252,
        encoding_rs::MACINTOSH,
        encoding_rs::SHIFT_JIS,
    ] {
        let (text, enc, had_errors) = encoding.decode(content);
        if !had_errors {
            return (text.into_owned(), enc);
        }
    }
    
    // Fallback to lossy UTF-8
    match String::from_utf8_lossy(content) {
        std::borrow::Cow::Borrowed(text) => (text.to_string(), encoding_rs::UTF_8),
        std::borrow::Cow::Owned(text) => (text, encoding_rs::UTF_8),
    }
}

fn decode_with_encoding(content: &[u8], encoding: &'static Encoding) -> (String, &'static Encoding) {
    let (text, _, _) = encoding.decode(content);
    (text.into_owned(), encoding)
}

fn glob_to_regex(pattern: &str) -> String {
    let mut regex = String::with_capacity(pattern.len() * 2);
    regex.push('^');
    
    for c in pattern.chars() {
        match c {
            '*' => regex.push_str(".*"),
            '?' => regex.push('.'),
            '.' => regex.push_str("\\."),
            '\\' => regex.push_str("\\\\"),
            '+' => regex.push_str("\\+"),
            '(' => regex.push_str("\\("),
            ')' => regex.push_str("\\)"),
            '[' => regex.push_str("\\["),
            ']' => regex.push_str("\\]"),
            '{' => regex.push_str("\\{"),
            '}' => regex.push_str("\\}"),
            '|' => regex.push_str("\\|"),
            '^' => regex.push_str("\\^"),
            '$' => regex.push_str("\\$"),
            _ => regex.push(c),
        }
    }
    
    regex.push('$');
    regex
} 
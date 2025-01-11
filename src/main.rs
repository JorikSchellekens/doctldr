use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use tracing::Level;
use tracing_subscriber::fmt;

mod config;
mod processing;
mod llm;
mod output;

use processing::DocumentProcessor;
use llm::{LlmSummarizer, OpenAiProvider};
use output::{OutputWriter, Summary};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Input directories to process
    #[arg(required = true)]
    input_dirs: Vec<PathBuf>,

    /// Output file path
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Output format (md, json, txt)
    #[arg(short, long)]
    format: Option<String>,

    /// LLM model to use
    #[arg(long)]
    model: Option<String>,

    /// Maximum tokens in summary
    #[arg(long)]
    max_tokens: Option<usize>,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Custom config file path
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Process without generating output
    #[arg(long)]
    dry_run: bool,

    /// Enable debug logging
    #[arg(long)]
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.debug {
        Level::DEBUG
    } else if cli.verbose {
        Level::INFO
    } else {
        Level::WARN
    };

    fmt::Subscriber::builder()
        .with_max_level(log_level)
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_file(false)
        .with_line_number(false)
        .with_level(true)
        .compact()
        .init();

    // Load configuration
    let mut config = config::Config::load()?;

    // Override config with CLI arguments
    if let Some(model) = cli.model {
        config.default.model = model;
    }
    if let Some(max_tokens) = cli.max_tokens {
        config.default.max_tokens = max_tokens;
    }
    if let Some(format) = cli.format.clone() {
        config.default.format = format;
    }
    config.default.verbose = cli.verbose;

    // Initialize components
    let document_processor = DocumentProcessor::new(
        config.processing.max_depth,
        config.processing.include_patterns.clone(),
        config.processing.exclude_patterns.clone(),
    );

    let llm_provider = OpenAiProvider::new(config.default.model.clone())?;
    let summarizer = LlmSummarizer::new(
        Box::new(llm_provider),
        config.default.max_tokens,
    );

    let output_writer = OutputWriter::new(&config.default.format)?;

    // Process each input directory
    let mut all_summaries = Vec::new();

    for dir in cli.input_dirs {
        // Process documents
        let documents = document_processor.process_directory(&dir).await?;

        // Generate summaries
        for document in documents {
            if cli.dry_run {
                println!("Would process: {}", document.path.display());
                continue;
            }

            let summary = summarizer.summarize(&document.content).await?;
            all_summaries.push(Summary::new(&document, summary));
        }
    }

    // Write output
    if !cli.dry_run {
        output_writer.write(all_summaries, cli.output.as_deref()).await?;
    }

    Ok(())
}

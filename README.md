# doctldr

A command-line tool that leverages Large Language Models (LLMs) to generate ultra-concise versions of documentation. Perfect for creating efficient context in LLM-powered workflows.

## Features

- Processes entire documentation directories
- Creates succinct, technically precise summaries
- Optimized for LLM context efficiency
- Preserves critical technical information while eliminating redundancy
- Supports multiple input formats (Markdown, RST, HTML, Plain Text)
- Multiple output formats (Markdown, JSON, Plain Text)
- Configurable via CLI or config file

## Installation

```bash
# Install using cargo
cargo install doctldr

# Or build from source
git clone https://github.com/yourusername/doctldr
cd doctldr
cargo build --release
```

## Quick Start

1. Set up your OpenAI API key:
```bash
export OPENAI_API_KEY='your-api-key'
```

2. Run the tool:
```bash
# Basic usage - summarize a directory
doctldr ./docs -o summary.md

# Process multiple directories
doctldr ./docs ./api-docs ./tutorials -o combined-summary.md

# Use a different output format
doctldr ./docs -f json -o summary.json
```

## Usage

```bash
doctldr [OPTIONS] <INPUT_DIRS>...

Arguments:
  <INPUT_DIRS>...  One or more input directories to process

Options:
  -o, --output <FILE>          Write output to FILE instead of stdout
  -f, --format <FORMAT>        Output format: md, json, txt [default: md]
  --model <MODEL>              LLM model to use [default: gpt-4]
  --max-tokens <NUMBER>        Maximum tokens in summary [default: 2048]
  -v, --verbose               Enable verbose output
  -c, --config <FILE>         Custom config file path
  --dry-run                   Process without generating output
  --debug                     Enable debug logging
  -h, --help                  Print help
```

## Configuration

The tool can be configured via a config file at `~/.config/doctldr/config.toml`:

```toml
[default]
model = "gpt-4"
max_tokens = 2048
format = "md"
verbose = false

[api]
provider = "openai"
key_env = "OPENAI_API_KEY"

[processing]
include_patterns = ["*.md", "*.rst", "*.txt", "*.html"]
exclude_patterns = ["node_modules", ".git"]
max_depth = 5

[output]
default_format = "md"
include_metadata = true
```

## Examples

### Basic Usage

```bash
# Summarize a project's documentation
doctldr ./docs -o summary.md

# Process multiple documentation sources
doctldr ./docs ./api-docs -o combined-summary.md
```

### Advanced Usage

```bash
# Use a different LLM model with custom token limit
doctldr ./docs --model gpt-3.5-turbo --max-tokens 1024

# Output as JSON with metadata
doctldr ./docs -f json -o summary.json

# Dry run to see what would be processed
doctldr ./docs --dry-run

# Enable debug logging
doctldr ./docs --debug -o summary.md
```

### Output Formats

The tool supports three output formats:

1. Markdown (default):
```markdown
# Summary of ./docs/api.md

API documentation summary...

---
_Compressed to 15.3% of original size_
```

2. JSON:
```json
{
  "summaries": [
    {
      "original_path": "./docs/api.md",
      "summary": "API documentation summary...",
      "metadata": {
        "original_size": 10240,
        "summary_size": 1568,
        "compression_ratio": 0.153
      }
    }
  ]
}
```

3. Plain Text:
```text
=== ./docs/api.md ===

API documentation summary...
```

## Design Philosophy

The tool is built around three core principles:

1. **Semantic Preservation**: All summaries maintain rigorous technical accuracy using precise terminology aimed at software development professionals.

2. **Intelligent Redundancy Elimination**: The summarization process actively filters out commonly known information (like basic setup procedures) that would be present in the target LLM's training data.

3. **Context Optimization**: Summaries are specifically formatted to serve as efficient context for other LLM workflows.

## License

MIT - See LICENSE file for details
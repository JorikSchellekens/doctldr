# doctldr Technical Specification

## Overview

doctldr is a Rust-based command-line tool designed to create ultra-concise documentation summaries using Large Language Models. The tool is specifically engineered to generate summaries that serve as efficient context for other LLM-powered workflows.

## Core Components

### 1. Document Processing

#### Input Handling
- **Supported Formats**
  - Markdown (.md)
  - reStructuredText (.rst)
  - HTML (.html, .htm)
  - Plain Text (.txt)
- **Directory Traversal**
  - Recursive processing with configurable depth
  - Glob pattern-based file filtering
  - UTF-8 and common encoding support
- **Content Extraction**
  - Intelligent markup stripping
  - Code block preservation
  - Hierarchical structure maintenance

#### Processing Pipeline
1. Directory scanning with pattern matching
2. File encoding detection and conversion
3. Format-specific content extraction
4. Text normalization and preprocessing

### 2. LLM Integration

#### Model Interface
- **Primary Support**
  - OpenAI GPT-4 (default)
  - OpenAI GPT-3.5-turbo
- **API Integration**
  - Async HTTP client
  - Rate limiting and error handling
  - Configurable timeouts
  - Retry mechanisms

#### Prompt Engineering
```text
System: You are a technical documentation summarizer. Your goal is to create ultra-concise summaries that preserve critical technical information while eliminating redundancy.

User: Create a concise technical summary of the following documentation. 
Focus on preserving critical technical information while removing redundant or commonly known details. 
Use precise technical terminology. The summary should be optimized for use as context in other LLM workflows.

<document content>
```

### 3. Summary Generation

#### Output Formats
1. **Markdown**
   ```markdown
   # Summary of path/to/doc.md
   
   Summary content...
   
   ---
   _Compressed to XX% of original size_
   ```

2. **JSON**
   ```json
   {
     "summaries": [{
       "original_path": "path/to/doc.md",
       "summary": "Summary content...",
       "metadata": {
         "original_size": 1000,
         "summary_size": 200,
         "compression_ratio": 0.2
       }
     }]
   }
   ```

3. **Plain Text**
   ```text
   === path/to/doc.md ===
   
   Summary content...
   ```

#### Summary Characteristics
- Maintains technical accuracy
- Eliminates redundant information
- Preserves critical implementation details
- Optimizes for token efficiency

## Configuration

### Global Configuration File
Location: `~/.config/doctldr/config.toml`

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

### CLI Interface

```bash
doctldr [OPTIONS] <INPUT_DIRS>...

Arguments:
  <INPUT_DIRS>...  One or more input directories to process

Options:
  -o, --output <FILE>          Output file path
  -f, --format <FORMAT>        Output format (md/json/txt)
  --model <MODEL>              LLM model selection
  --max-tokens <NUMBER>        Token limit for summaries
  -v, --verbose               Enable verbose output
  -c, --config <FILE>         Custom config file
  --dry-run                   Process without output
  --debug                     Enable debug logging
```

## Processing Pipeline

### 1. Input Processing
```rust
pub async fn process_directory(&self, dir: &Path) -> Result<Vec<Document>> {
    // Directory traversal with configured patterns
    // File encoding detection
    // Format-specific parsing
}
```

### 2. LLM Processing
```rust
pub async fn summarize(&self, content: &str) -> Result<String> {
    // Prompt construction
    // API call with retry logic
    // Response parsing and validation
}
```

### 3. Output Generation
```rust
pub async fn write(&self, summaries: Vec<Summary>, output_path: Option<&Path>) -> Result<()> {
    // Format selection
    // Content formatting
    // File or stdout output
}
```

## Error Handling

### Categories
1. **Input Errors**
   - Invalid file encodings
   - Unsupported file formats
   - Permission issues

2. **API Errors**
   - Rate limiting
   - Token limits
   - Network issues
   - Authentication failures

3. **Output Errors**
   - File system issues
   - Format conversion errors

### Recovery Mechanisms
- Automatic retries for transient failures
- Graceful degradation for unsupported features
- Detailed error reporting
- Debug logging for troubleshooting

## Performance Considerations

### Optimization Strategies
1. **Parallel Processing**
   - Concurrent file processing
   - Async I/O operations
   - Batch API requests

2. **Memory Management**
   - Streaming large files
   - Buffer optimization
   - Efficient string handling

3. **API Efficiency**
   - Request batching
   - Response caching
   - Token optimization

## Security

### API Key Management
- Environment variable based
- No key storage in config files
- Secure key transmission

### Input Validation
- Path traversal prevention
- File type verification
- Content sanitization

### Output Safety
- Safe file handling
- Permission checking
- Secure temporary files

## Extension Points

### Custom Components
1. **LLM Providers**
   - Alternative API providers
   - Local model support
   - Custom prompt engineering

2. **Output Formats**
   - New format implementations
   - Custom templates
   - Metadata extensions

3. **Processing Pipeline**
   - Custom preprocessors
   - Additional file formats
   - Filter implementations 
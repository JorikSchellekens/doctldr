use anyhow::{Result, Context};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

#[async_trait]
pub trait LlmProvider {
    async fn summarize(&self, content: &str, max_tokens: usize) -> Result<String>;
}

pub struct OpenAiProvider {
    client: Client,
    api_key: String,
    model: String,
}

#[derive(Debug, Serialize)]
struct OpenAiRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    max_tokens: usize,
    temperature: f32,
}

#[derive(Debug, Serialize)]
struct OpenAiMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAiResponse {
    choices: Vec<OpenAiChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAiChoice {
    message: OpenAiResponseMessage,
}

#[derive(Debug, Deserialize)]
struct OpenAiResponseMessage {
    content: String,
}

impl OpenAiProvider {
    pub fn new(model: String) -> Result<Self> {
        let api_key = env::var("OPENAI_API_KEY")
            .context("OPENAI_API_KEY environment variable not found")?;

        Ok(Self {
            client: Client::new(),
            api_key,
            model,
        })
    }

    fn create_summary_prompt(content: &str) -> String {
        format!(
            "Create a technical summary optimized for an LLM to understand how to use and implement this tool/feature. Focus on:
1. Function signatures, types, and interfaces
2. Concrete usage examples with actual parameters
3. Key implementation details and data structures
4. API endpoints and their request/response formats
5. Configuration options with specific valid values
6. Command-line usage patterns with real examples

Exclude:
- General descriptions without technical details
- Marketing or promotional content
- Basic setup instructions unless they contain specific commands
- Conceptual explanations without code or concrete examples

Format the response to maximize information density while maintaining clear structure.
If the documentation contains code examples, preserve them with their context.

Documentation to summarize:

{}",
            content
        )
    }
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    async fn summarize(&self, content: &str, max_tokens: usize) -> Result<String> {
        let request = OpenAiRequest {
            model: self.model.clone(),
            messages: vec![
                OpenAiMessage {
                    role: "system".to_string(),
                    content: "You are a technical documentation processor focused on creating summaries for LLM consumption. \
                    Your goal is to extract and preserve implementation details, concrete examples, and technical specifications \
                    while eliminating general descriptions and conceptual explanations. Prioritize code examples, API specifications, \
                    and exact usage patterns. Format your responses to maximize information density for LLM parsing.".to_string(),
                },
                OpenAiMessage {
                    role: "user".to_string(),
                    content: Self::create_summary_prompt(content),
                },
            ],
            max_tokens,
            temperature: 0.1,
        };

        let response = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await?
            .json::<OpenAiResponse>()
            .await?;

        response.choices
            .first()
            .map(|choice| choice.message.content.clone())
            .context("No response from OpenAI API")
    }
}

pub struct LlmSummarizer {
    provider: Box<dyn LlmProvider + Send + Sync>,
    max_tokens: usize,
}

impl LlmSummarizer {
    pub fn new(provider: Box<dyn LlmProvider + Send + Sync>, max_tokens: usize) -> Self {
        Self {
            provider,
            max_tokens,
        }
    }

    pub async fn summarize(&self, content: &str) -> Result<String> {
        self.provider.summarize(content, self.max_tokens).await
    }
} 
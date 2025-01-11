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
            "Create a concise technical summary of the following documentation. \
            Focus on preserving critical technical information while removing redundant or commonly known details. \
            Use precise technical terminology. The summary should be optimized for use as context in other LLM workflows.\n\n{}",
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
                    content: "You are a technical documentation summarizer. Your goal is to create ultra-concise summaries that preserve critical technical information while eliminating redundancy.".to_string(),
                },
                OpenAiMessage {
                    role: "user".to_string(),
                    content: Self::create_summary_prompt(content),
                },
            ],
            max_tokens,
            temperature: 0.3,
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
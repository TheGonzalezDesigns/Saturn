pub mod search {
    use anyhow::{bail, Error, Result};
    use dotenv::dotenv;
    use reqwest::{header, Client, StatusCode};
    use serde::{Deserialize, Serialize};
    use std::env;

    #[derive(Serialize)]
    pub struct PerplexityPayloadMessage {
        role: String,
        content: String,
    }

    impl PerplexityPayloadMessage {
        pub fn new(role: String, content: String) -> Self {
            PerplexityPayloadMessage { role, content }
        }
    }

    pub type PerplexityPayloadMessages = Vec<PerplexityPayloadMessage>;

    #[derive(Serialize)]
    pub struct PerplexityPayload {
        model: String,
        messages: PerplexityPayloadMessages,
        max_tokens: Option<u32>,
        temperature: f32,
        top_p: f32,
        return_citations: bool,
        search_domain_filter: Vec<String>,
        return_images: bool,
        return_related_questions: bool,
        search_recency_filter: String,
        top_k: u32,
        stream: bool,
        presence_penalty: f32,
        frequency_penalty: f32,
    }

    impl PerplexityPayload {
        pub fn new(model: String, query: String) -> Self {
            let system_message = PerplexityPayloadMessage::new(
                "system".to_string(),
                "Be precise and concise.".to_string(),
            );
            let user_message = PerplexityPayloadMessage::new("user".to_string(), query);
            let messages = vec![system_message, user_message];
            PerplexityPayload {
                model,
                messages,
                max_tokens: None,
                temperature: 0.2,
                top_p: 0.9,
                return_citations: true,
                search_domain_filter: vec!["perplexity.ai".to_string()],
                return_images: false,
                return_related_questions: false,
                search_recency_filter: "month".to_string(),
                top_k: 0,
                stream: false,
                presence_penalty: 0.0,
                frequency_penalty: 1.0,
            }
        }
    }

    #[derive(Deserialize)]
    pub struct PerplexityResponseChoiceMessage {
        content: String,
    }

    #[derive(Deserialize)]
    pub struct PerplexityResponseChoice {
        message: PerplexityResponseChoiceMessage,
    }

    pub type PerplexityResponseChoices = Vec<PerplexityResponseChoice>;

    #[derive(Deserialize)]
    pub struct PerplexityResponse {
        choices: PerplexityResponseChoices,
    }

    pub async fn search(query: String) -> Result<String, Error> {
        dotenv().ok();
        let perplexity_api_key = env::var("PERPLEXITY_API_KEY").expect("Failed to extract PERPLEXITY_API_KEY");
        let client = Client::new();
        let payload = PerplexityPayload::new("llama-3.1-sonar-small-128k-online".to_string(), query);

        let response = client
            .post("https://api.perplexity.ai/chat/completions")
            .header(header::CONTENT_TYPE, "application/json; charset=utf-8")
            .header(header::AUTHORIZATION, format!("Bearer {}", perplexity_api_key))
            .json(&payload)
            .send()
            .await?;

        if response.status() != StatusCode::OK {
            bail!("Failed to fetch response: {}", response.status());
        }

        let completion: PerplexityResponse = response.json().await?;

        if let Some(choice) = completion.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            bail!("No choices found in the response.")
        }
    }
}

pub mod openai {
    use anyhow::{bail, Error, Result};
    use dotenv::dotenv;
    use reqwest::{header, Client, StatusCode};
    use serde::{Deserialize, Serialize};
    use std::env;

    #[derive(Serialize)]
    pub struct OpenAIPayloadMessage {
        role: String,
        content: String,
    }

    impl OpenAIPayloadMessage {
        pub fn new(role: String, content: String) -> Self {
            OpenAIPayloadMessage { role, content }
        }
    }

    pub type OpenAIPayloadMessages = Vec<OpenAIPayloadMessage>;

    #[derive(Serialize)]
    pub struct OpenAIPayload {
        model: String,
        messages: OpenAIPayloadMessages,
    }

    impl OpenAIPayload {
        pub fn new(model: String, query: String) -> Self {
            let system_message = OpenAIPayloadMessage::new(
                "system".to_string(),
                "You are a helpful assistant.".to_string(),
            );
            let user_message = OpenAIPayloadMessage::new("user".to_string(), query);
            let messages = vec![system_message, user_message];
            OpenAIPayload { model, messages }
        }
    }

    #[derive(Deserialize)]
    pub struct OpenAIChatCompletionResponseChoiceMessage {
        content: String,
    }

    #[derive(Deserialize)]
    pub struct OpenAIChatCompletionResponseChoice {
        message: OpenAIChatCompletionResponseChoiceMessage,
    }

    pub type OpenAIChatCompletionResponseChoices = Vec<OpenAIChatCompletionResponseChoice>;

    #[derive(Deserialize)]
    pub struct OpenAIChatCompletionResponse {
        choices: OpenAIChatCompletionResponseChoices,
    }

    pub async fn openai(query: String) -> Result<String, Error> {
        dotenv().ok();
        let openai_api_key = env::var("OPENAI_API_KEY").expect("Failed to extract OPENAI_API_KEY");
        let client = Client::new();
        let payload = OpenAIPayload::new("gpt-4o".to_string(), query);

        let response = client
            .post("https://api.openai.com/v1/chat/completions")
            .header(header::CONTENT_TYPE, "application/json; charset=utf-8")
            .header(header::AUTHORIZATION, format!("Bearer {}", openai_api_key))
            .json(&payload)
            .send()
            .await?;

        if response.status() != StatusCode::OK {
            bail!("Failed to fetch response: {}", response.status());
        }

        let completion: OpenAIChatCompletionResponse = response.json().await?;

        // Extract the content from the first choice
        if let Some(choice) = completion.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            bail!("No choices found in the response.")
        }
    }
}

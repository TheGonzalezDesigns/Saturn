pub mod openai {
    use anyhow::{bail, Error, Result};
    use dotenv::dotenv;
    use reqwest::{header, Client, StatusCode};
    use serde::{Deserialize, Serialize};
    use std::env;

    #[derive(Serialize, Debug)]
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

    #[derive(Serialize, Debug)]
    pub struct OpenAIFunctionParameter {
        #[serde(rename = "type")]
        param_type: String,
        properties: serde_json::Value,
        required: Vec<String>,
    }

    #[derive(Serialize, Debug)]
    pub struct OpenAIFunctionDetails {
        name: String,
        description: String,
        parameters: OpenAIFunctionParameter,
    }

    #[derive(Serialize, Debug)]
    pub struct OpenAIFunction {
        r#type: String,
        function: OpenAIFunctionDetails,
    }

    #[derive(Serialize, Debug)]
    pub struct OpenAIFunctionPayload {
        model: String,
        messages: OpenAIPayloadMessages,
        tools: Vec<OpenAIFunction>,
        tool_choice: String,
    }

    impl OpenAIFunctionPayload {
        pub fn new(
            model: String,
            query: String,
            function_name: String,
            function_description: String,
            properties: serde_json::Value,
            required: Vec<String>,
        ) -> Self {
            let system_message = OpenAIPayloadMessage::new(
                "system".to_string(),
                "You are a helpful assistant.".to_string(),
            );
            let user_message = OpenAIPayloadMessage::new("user".to_string(), query);
            let messages = vec![system_message, user_message];

            let parameters = OpenAIFunctionParameter {
                param_type: "object".to_string(),
                properties,
                required,
            };

            let function_details = OpenAIFunctionDetails {
                name: function_name,
                description: function_description,
                parameters,
            };

            let tool = OpenAIFunction {
                r#type: "function".to_string(),
                function: function_details,
            };

            OpenAIFunctionPayload {
                model,
                messages,
                tools: vec![tool],
                tool_choice: "auto".to_string(),
            }
        }
    }

    #[derive(Deserialize)]
    pub struct OpenAIChatCompletionResponseChoiceMessageToolCall {
        id: String,
        r#type: String,
        function: FunctionCallDetails,
    }

    #[derive(Deserialize)]
    pub struct FunctionCallDetails {
        name: String,
        arguments: String,
    }

    #[derive(Deserialize)]
    pub struct OpenAIChatCompletionResponseChoiceMessage {
        content: Option<String>,
        tool_calls: Option<Vec<OpenAIChatCompletionResponseChoiceMessageToolCall>>,
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

    // Basic completion method
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
            if let Some(content) = &choice.message.content {
                return Ok(content.clone());
            }
        }

        bail!("No valid response content found.")
    }

    // Flexible function call method
    pub async fn function_call(
        query: String,
        function_name: String,
        function_description: String,
        properties: serde_json::Value,
        required: Vec<String>,
    ) -> Result<String, Error> {
        dotenv().ok();
        let openai_api_key = env::var("OPENAI_API_KEY").expect("Failed to extract OPENAI_API_KEY");
        let client = Client::new();

        let payload = OpenAIFunctionPayload::new(
            "gpt-4o".to_string(),
            query,
            function_name,
            function_description,
            properties,
            required,
        );

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

        // Check for tool calls
        if let Some(choice) = completion.choices.first() {
            if let Some(tool_calls) = &choice.message.tool_calls {
                if let Some(tool_call) = tool_calls.first() {
                    return Ok(format!(
                        "Function '{}' called with arguments: {}",
                        tool_call.function.name, tool_call.function.arguments
                    ));
                }
            }

            // If no tool calls, return the content if available
            if let Some(content) = &choice.message.content {
                return Ok(content.clone());
            }
        }

        bail!("No valid response or tool call found in the response.")
    }
}

#[cfg(test)]
mod tests {
    use super::openai::*;
    use std::env;
    use dotenv::dotenv;
    use serde_json::json;

    // Helper function to set up the environment variable
    fn setup_openai_key() {
        dotenv().ok();
        if env::var("OPENAI_API_KEY").is_err() {
            panic!("OPENAI_API_KEY is not set in the environment");
        }
    }

    #[tokio::test]
    async fn test_openai_basic_completion() {
        setup_openai_key();

        // Test a simple query
        let query = "What is the capital of France?".to_string();
        match openai(query).await {
            Ok(response) => {
                assert!(!response.is_empty(), "Expected a non-empty response");
                println!("Basic Completion Response: {}", response);
            }
            Err(e) => panic!("Failed to get a response from OpenAI: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_openai_function_call() {
        setup_openai_key();

        // Define dynamic properties and required fields for the function call
        let properties = json!({
            "location": {
                "type": "string",
                "description": "The city and state, e.g., San Francisco, CA"
            },
            "unit": {
                "type": "string",
                "enum": ["celsius", "fahrenheit"]
            }
        });
        let required = vec!["location".to_string()];

        // Test a query that may trigger a function call
        let query = "What's the weather like in Boston today?".to_string();
        let function_name = "get_current_weather".to_string();
        let function_description = "Get the current weather in a given location".to_string();

        match function_call(query, function_name, function_description, properties, required).await {
            Ok(response) => {
                assert!(
                    response.contains("Function") || !response.is_empty(),
                    "Expected a tool call or valid response content"
                );
                println!("Function Call Response: {}", response);
            }
            Err(e) => panic!("Failed to get a response from OpenAI function call: {:?}", e),
        }
    }
}

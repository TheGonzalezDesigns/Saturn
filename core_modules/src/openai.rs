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

pub mod openai_json {
    use anyhow::{bail, Error, Result};
    use dotenv::dotenv;
    use reqwest::{header, Client, StatusCode};
    use serde::{Deserialize, Serialize};
    use serde_json::{json, Value};
    use std::env;

    #[derive(Serialize, Debug)]
    pub struct OpenAIPayloadMessage {
        role: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        content: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        function_call: Option<Value>,
    }

    impl OpenAIPayloadMessage {
        pub fn new(
            role: String,
            content: Option<String>,
            function_call: Option<Value>,
            name: Option<String>,
        ) -> Self {
            OpenAIPayloadMessage {
                role,
                content,
                function_call,
                name,
            }
        }
    }

    pub type OpenAIPayloadMessages = Vec<OpenAIPayloadMessage>;

    #[derive(Serialize, Debug)]
    pub struct OpenAIFunctionParameter {
        #[serde(rename = "type")]
        param_type: String,
        properties: Value,
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
            user_query: String,
            function_name: String,
            function_description: String,
            properties: Value,
            required: Vec<String>,
            function_call_arguments: Value,
        ) -> Self {
            let user_message =
                OpenAIPayloadMessage::new("user".to_string(), Some(user_query), None, None);

            let assistant_message = OpenAIPayloadMessage::new(
                "assistant".to_string(),
                None,
                Some(
                    json!({ "name": function_name, "arguments": serde_json::to_string(&function_call_arguments).unwrap() }),
                ),
                None,
            );

            let function_message = OpenAIPayloadMessage::new(
                "function".to_string(),
                Some(serde_json::to_string(&function_call_arguments).unwrap()),
                None,
                Some(function_name.clone()),
            );

            let messages = vec![user_message, assistant_message, function_message];

            let parameters = OpenAIFunctionParameter {
                param_type: "object".to_string(),
                properties,
                required,
            };

            let function_details = OpenAIFunctionDetails {
                name: function_name.clone(),
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

    #[derive(Deserialize, Debug)]
    pub struct OpenAIChatCompletionResponseChoiceMessageToolCall {
        id: String,
        r#type: String,
        function: FunctionCallDetails,
    }

    #[derive(Deserialize, Debug)]
    pub struct FunctionCallDetails {
        name: String,
        arguments: String,
    }

    #[derive(Deserialize, Debug)]
    pub struct OpenAIChatCompletionResponseChoiceMessage {
        content: Option<String>,
        tool_calls: Option<Vec<OpenAIChatCompletionResponseChoiceMessageToolCall>>,
    }

    #[derive(Deserialize, Debug)]
    pub struct OpenAIChatCompletionResponseChoice {
        message: OpenAIChatCompletionResponseChoiceMessage,
    }

    pub type OpenAIChatCompletionResponseChoices = Vec<OpenAIChatCompletionResponseChoice>;

    #[derive(Deserialize, Debug)]
    pub struct OpenAIChatCompletionResponse {
        pub choices: OpenAIChatCompletionResponseChoices,
    }

    // Flexible function call method
    pub async fn function_call(
        query: String,
        function_name: String,
        function_description: String,
        properties: Value,
        required: Vec<String>,
        function_call_arguments: Value,
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
            function_call_arguments,
        );

        let pretty_payload = serde_json::to_string_pretty(&payload).map_err(|e| Error::new(e))?;
        //bail!("Payload:\n{}", pretty_payload);

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

        println!("Response: {:#?}", completion);

        if let Some(choice) = completion.choices.first() {
            if let Some(tool_calls) = &choice.message.tool_calls {
                if let Some(tool_call) = tool_calls.first() {
                    return Ok(format!(
                        "Function '{}' called with arguments: {}",
                        tool_call.function.name, tool_call.function.arguments
                    ));
                }
            }

            if let Some(content) = &choice.message.content {
                return Ok(content.clone());
            }
        }

        bail!("No valid response or tool call found in the response.")
    }
}

#[cfg(test)]
mod tests {
    use super::openai_json::*;
    use dotenv::dotenv;
    use serde_json::json;
    use std::env;

    fn setup_openai_key() {
        dotenv().ok();
        if env::var("OPENAI_API_KEY").is_err() {
            panic!("OPENAI_API_KEY is not set in the environment");
        }
    }

    #[tokio::test]
    async fn test_openai_function_call_with_election_data() {
        setup_openai_key();

        let properties = json!({
            "candidate": {
                "type": "string",
                "description": "The name of the candidate"
            },
           "status": {
             "type": "string",
             "enum": ["loser", "winner"]
           },
           "votes": {
             "type": "integer",
             "description": "amount of popular votes percentage_wise"
           }
        });
        let required = vec![
            "candidate".to_string(),
            "status".to_string(),
            "votes".to_string(),
        ];

        let function_call_arguments = json!({
            "election_year": 2020,
        });

        let query = "Who won the last USA presidential election?".to_string();
        let function_name = "get_previous_candidate".to_string();
        let function_description =
            "Get the previous winner of the latest USA elections".to_string();

        match function_call(
            query,
            function_name,
            function_description,
            properties,
            required,
            function_call_arguments,
        )
        .await
        {
            Ok(response) => {
                assert!(
                    response.contains("Function") || !response.is_empty(),
                    "Expected a tool call or valid response content"
                );
                println!("Function Call Response with Election Data: {}", response);
            }
            Err(e) => panic!(
                "Failed to get a response from OpenAI function call: {:?}",
                e
            ),
        }
    }

    #[tokio::test]
    async fn test_openai_function_call_with_location_data_flat() {
        setup_openai_key();

        // Define flat properties for location data (no nested structures)
        let properties = json!({
            "latitude": {
                "type": "number",
                "description": "Latitude of the location"
            },
            "longitude": {
                "type": "number",
                "description": "Longitude of the location"
            },
            "is_valid_json_response": {
                "type": "boolean",
                "enum": ["true", "false"],
                "description": "Is this response valid json."
            }
        });
        let required = vec![
            "latitude".to_string(),
            "longitude".to_string(),
            "is_valid_json_response".to_string(),
        ];

        // Define arguments for the function call
        let function_call_arguments = json!({
            "city": "Havana",
            "country": "Cuba"
        });

        // Test query to retrieve location data
        let query = "What are the coordinates of Havana, Cuba?".to_string();
        let function_name = "get_location_coordinates".to_string();
        let function_description =
            "Get the latitude and longitude of a specified city and country.".to_string();

        match function_call(
            query,
            function_name,
            function_description,
            properties,
            required,
            function_call_arguments,
        )
        .await
        {
            Ok(response) => {
                assert!(
                    response.contains("Function") || !response.is_empty(),
                    "Expected a tool call or valid response content"
                );
                println!("Function Call Response with Location Data: {}", response);
                panic!("yeah!");
            }
            Err(e) => panic!(
                "Failed to get a response from OpenAI function call: {:?}",
                e
            ),
        }
    }
}

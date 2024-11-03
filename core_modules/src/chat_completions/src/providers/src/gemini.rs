pub mod gemini {
    use anyhow::{bail, Error, Result};
    use dotenv::dotenv;
    use reqwest::{header, Client, StatusCode};
    use serde::{Deserialize, Serialize};
    use std::{env, thread, time};

    #[derive(Serialize)]
    struct GeminiRequest {
        contents: Vec<GeminiContent>,
    }

    #[derive(Serialize)]
    struct GeminiContent {
        parts: Vec<GeminiPart>,
    }

    #[derive(Serialize)]
    struct GeminiPart {
        text: String,
    }

    #[derive(Deserialize)]
    struct GeminiResponse {
        candidates: Vec<GeminiCandidate>,
    }

    #[derive(Deserialize)]
    struct GeminiCandidate {
        content: GeminiCandidateContent,
    }

    #[derive(Deserialize)]
    struct GeminiCandidateContent {
        parts: Vec<GeminiCandidatePart>,
    }

    #[derive(Deserialize)]
    struct GeminiCandidatePart {
        text: String,
    }

    pub async fn gemini(query: String) -> Result<String, Error> {
        dotenv().ok();
        let api_key = env::var("GEMINI_API_KEY").expect("Failed to extract GEMINI_API_KEY");
        let url = format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-pro:generateContent?key={}", api_key);

        let client = Client::new();
        let mut valid_response = false;
        let delay = time::Duration::from_secs(1);

        while !valid_response {
            // Build the JSON request payload
            let payload = GeminiRequest {
                contents: vec![GeminiContent {
                    parts: vec![GeminiPart {
                        text: query.clone(),
                    }],
                }],
            };

            // Send the POST request
            let response = client
                .post(&url)
                .header(header::CONTENT_TYPE, "application/json")
                .json(&payload)
                .send()
                .await?;

            if response.status() != StatusCode::OK {
                bail!("Failed to fetch response: {}", response.status());
            }

            // Parse the JSON response
            let result: GeminiResponse = response.json().await?;

            if let Some(candidate) = result.candidates.first() {
                if let Some(part) = candidate.content.parts.first() {
                    if !part.text.is_empty() && part.text != "null" {
                        valid_response = true;
                        return Ok(part.text.clone());
                    }
                }
            }

            // Wait before retrying
            thread::sleep(delay);
        }

        bail!("Failed to retrieve a valid response after retries.")
    }
}

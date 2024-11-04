pub mod saturn {
    use crate::chat_completions::providers::{
        gemini::gemini::gemini, openai::openai::openai, perplexity::perplexity::perplexity,
    };
    use crate::chat_completions::utils::needs_internet::needs_internet::needs_internet;
    use anyhow::Result;

    /// Saturn bot: Receives a query and tries to fulfill it using OpenAI first.
    /// If OpenAI cannot fulfill the query, it falls back to Gemini.
    /// If internet access is needed, it sends the query to Perplexity.
    ///
    /// # Arguments
    /// * `query` - A `String` representing the user query.
    ///
    /// # Returns
    /// * `Result<String>` - The response from OpenAI, Gemini, or Perplexity based on the internet check.
    pub async fn saturn(query: String) -> Result<String> {
        // Step 1: Try to fulfill the query using OpenAI
        let mut response = match openai(query.clone()).await {
            Ok(res) => res,
            Err(_) => {
                eprintln!("OpenAI failed; falling back to Gemini.");
                // Step 2: If OpenAI fails, try using Gemini
                match gemini(query.clone()).await {
                    Ok(res) => res,
                    Err(_) => {
                        eprintln!("Gemini also failed; no response generated.");
                        "".to_string() // If both fail, return an empty string as a last resort
                    }
                }
            }
        };

        // Step 3: Check if the response requires internet access
        if needs_internet(query.clone()).await? {
            //println!("Internet access is required; querying Perplexity.");
            // Step 4: If internet access is required, use Perplexity
            response = perplexity(query).await?;
        }

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::saturn::saturn;

    #[tokio::test]
    async fn test_saturn_bot() {
        let query = "What is the current state of the cryptocurrency market?".to_string();

        match saturn(query.clone()).await {
            Ok(response) => {
                println!("Saturn bot response: {}", response);
                assert!(
                    !response.is_empty(),
                    "Saturn bot should return a valid response."
                );
            }
            Err(e) => panic!("Saturn bot failed: {:?}", e),
        }
    }
}

use regex::Regex;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use serde_json::Value;

#[derive(Debug, Deserialize)]
struct Candidate {
    content: Content,
}

#[derive(Debug, Deserialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Debug, Deserialize)]
struct Part {
    text: String,
}

#[derive(Debug, Deserialize)]
struct Response {
    candidates: Vec<Candidate>,
}

/// Parses the input JSON string to extract the image summary and keywords.
///
/// # Arguments
/// * `json_string` - A string slice containing the JSON data.
///
/// # Returns
/// A `Result` containing a tuple of `(Option<String>, Option<Vec<String>>)` on success,
/// where the first element is the summary and the second is a vector of keywords.
/// Returns a `String` error message on failure.
pub fn parse_summary_and_keywords(json_data: &str) -> Option<(String, Vec<String>)> {
    let response: Response = serde_json::from_str(json_data).ok()?;
    let part_text = &response.candidates.get(0)?.content.parts.get(0)?.text;

    // Extract JSON block inside the ```json ... ``` section
    let re_json = Regex::new(r"```json\s*(\{[\s\S]*?\})\s*```").ok()?;
    let json_caps = re_json.captures(part_text)?;
    let json_blob = json_caps.get(1)?.as_str();

    let inner_json: Value = serde_json::from_str(json_blob).ok()?;
    let summary = inner_json.get("CONCISESUMMARY")?.as_str()?.to_string();

    let keywords_json = inner_json.get("KEYWORDS")?.as_array()?;
    let keywords: Vec<String> = keywords_json
        .iter()
        .filter_map(|val| val.as_str().map(|s| s.to_string()))
        .collect();

    Some((summary, keywords))
}
pub async fn send_image_to_gemini_api(
    encoded_image: &str,
    mime_type: &str,
) -> Result<(String, Vec<String>), reqwest::Error> {
    let api_key = "AIzaSyBzxccI9t-fC6V5qFoEy0ntwdb4D_Ray8c";
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key={}",
        api_key
    );
    let prompt_text = "I am giving you a image give me a json with concise summary of this image starting with CONCISESUMMARY: and then give me 100 keywords to search for this image starting with KEYWORDS:";

    let body = json!({
        "contents": [
            {
                "parts": [
                    {
                        "inline_data": {
                            "mime_type": mime_type,
                            "data": encoded_image
                        }
                    },
                    {
                        "text": prompt_text
                    }
                ]
            }
        ]
    });

    let client = Client::new();
    let res = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    let response_text = res.text().await?;
    println!("Response: {}", response_text);
    let (summary, keywords) = parse_summary_and_keywords(&response_text).unwrap_or_default();
    println!("{:?}\n{:?}", summary, keywords);
    Ok((summary, keywords))
}

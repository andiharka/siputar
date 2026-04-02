use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::time::{Duration, Instant};

const BASE_URL: &str = "https://api.elevenlabs.io/v1";

// Rate limiter: 1 request per second
pub struct RateLimiter {
    last_request: Mutex<Option<Instant>>,
    min_interval: Duration,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            last_request: Mutex::new(None),
            min_interval: Duration::from_secs(1),
        }
    }

    pub async fn wait(&self) {
        let wait_duration = {
            let last = self.last_request.lock().unwrap();
            if let Some(last_time) = *last {
                let elapsed = last_time.elapsed();
                if elapsed < self.min_interval {
                    Some(self.min_interval - elapsed)
                } else {
                    None
                }
            } else {
                None
            }
        };

        if let Some(duration) = wait_duration {
            tokio::time::sleep(duration).await;
        }

        *self.last_request.lock().unwrap() = Some(Instant::now());
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

// API Response Types
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserSubscription {
    pub character_count: i64,
    pub character_limit: i64,
    pub tier: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_character_count_reset_unix: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Voice {
    pub voice_id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VoicesResponse {
    pub voices: Vec<Voice>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Language {
    pub language_id: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Model {
    pub model_id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub can_be_finetuned: bool,
    pub can_do_text_to_speech: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub languages: Option<Vec<Language>>,
    #[serde(default)]
    pub max_characters_request_free_user: i64,
    #[serde(default)]
    pub max_characters_request_subscribed_user: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VoiceSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stability: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub similarity_boost: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_speaker_boost: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HistoryItem {
    pub history_item_id: String,
    pub request_id: String,
    pub voice_id: String,
    pub voice_name: String,
    pub model_id: String,
    pub text: String,
    pub date_unix: i64,
    pub character_count_change_from: i64,
    pub character_count_change_to: i64,
    pub state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<VoiceSettings>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HistoryResponse {
    pub history: Vec<HistoryItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_history_item_id: Option<String>,
    pub has_more: bool,
}

#[derive(Debug, Serialize)]
pub struct TTSRequest {
    pub text: String,
    pub model_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice_settings: Option<VoiceSettings>,
}

// API Client
pub struct ElevenLabsClient {
    client: Client,
    pub rate_limiter: RateLimiter,
}

impl ElevenLabsClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            rate_limiter: RateLimiter::new(),
        }
    }

    fn get_api_key() -> Result<String, String> {
        crate::keychain::get_api_key()?
            .ok_or_else(|| "API key not configured".to_string())
    }

    pub async fn test_api_key(&self, key: &str) -> Result<bool, String> {
        let response = self.client
            .get(format!("{}/user", BASE_URL))
            .header("xi-api-key", key)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        Ok(response.status().is_success())
    }

    pub async fn get_subscription(&self) -> Result<UserSubscription, String> {
        self.rate_limiter.wait().await;
        let api_key = Self::get_api_key()?;

        let response = self.client
            .get(format!("{}/user/subscription", BASE_URL))
            .header("xi-api-key", &api_key)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("API error: {}", response.status()));
        }

        response.json::<UserSubscription>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    pub async fn get_voices(&self) -> Result<Vec<Voice>, String> {
        self.rate_limiter.wait().await;
        let api_key = Self::get_api_key()?;

        let response = self.client
            .get(format!("{}/voices", BASE_URL))
            .header("xi-api-key", &api_key)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("API error: {}", response.status()));
        }

        let voices_response: VoicesResponse = response.json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(voices_response.voices)
    }

    pub async fn get_models(&self) -> Result<Vec<Model>, String> {
        self.rate_limiter.wait().await;
        let api_key = Self::get_api_key()?;

        let response = self.client
            .get(format!("{}/models", BASE_URL))
            .header("xi-api-key", &api_key)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("API error: {}", response.status()));
        }

        response.json::<Vec<Model>>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    pub async fn get_history(&self, page_size: Option<u32>, start_after: Option<String>) -> Result<HistoryResponse, String> {
        self.rate_limiter.wait().await;
        let api_key = Self::get_api_key()?;

        let mut url = format!("{}/history", BASE_URL);
        let mut params = vec![];
        
        if let Some(size) = page_size {
            params.push(format!("page_size={}", size));
        }
        if let Some(start) = start_after {
            params.push(format!("start_after_history_item_id={}", start));
        }
        
        if !params.is_empty() {
            url = format!("{}?{}", url, params.join("&"));
        }

        let response = self.client
            .get(&url)
            .header("xi-api-key", &api_key)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("API error: {}", response.status()));
        }

        response.json::<HistoryResponse>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    pub async fn generate_speech(
        &self,
        voice_id: &str,
        text: &str,
        model_id: &str,
        language: Option<&str>,
        stability: Option<f64>,
        similarity_boost: Option<f64>,
        speed: Option<f64>,
    ) -> Result<Vec<u8>, String> {
        self.rate_limiter.wait().await;
        let api_key = Self::get_api_key()?;

        // All voice settings (stability, similarity_boost, speed) go in the
        // voice_settings body — speed is NOT a query parameter.
        let voice_settings = if stability.is_some() || similarity_boost.is_some() || speed.is_some() {
            Some(VoiceSettings {
                stability,
                similarity_boost,
                style: None,
                use_speaker_boost: None,
                speed,
            })
        } else {
            None
        };

        let request = TTSRequest {
            text: text.to_string(),
            model_id: model_id.to_string(),
            language_code: language.map(|s| s.to_string()),
            voice_settings,
        };

        let url = format!("{}/text-to-speech/{}?output_format=mp3_44100_128", BASE_URL, voice_id);

        let response = self.client
            .post(&url)
            .header("xi-api-key", &api_key)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("API error: {}", error_text));
        }

        response.bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(|e| format!("Failed to read audio data: {}", e))
    }

    pub async fn download_audio(&self, history_item_id: &str) -> Result<Vec<u8>, String> {
        self.rate_limiter.wait().await;
        let api_key = Self::get_api_key()?;

        let response = self.client
            .get(format!("{}/history/{}/audio", BASE_URL, history_item_id))
            .header("xi-api-key", &api_key)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("API error: {}", response.status()));
        }

        response.bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(|e| format!("Failed to read audio data: {}", e))
    }

    pub async fn delete_history_item(&self, history_item_id: &str) -> Result<(), String> {
        self.rate_limiter.wait().await;
        let api_key = Self::get_api_key()?;

        let response = self.client
            .delete(format!("{}/history/{}", BASE_URL, history_item_id))
            .header("xi-api-key", &api_key)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("API error: {}", response.status()));
        }

        Ok(())
    }
}

impl Default for ElevenLabsClient {
    fn default() -> Self {
        Self::new()
    }
}

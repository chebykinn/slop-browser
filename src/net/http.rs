use super::loader::LoadError;
use reqwest::blocking::Client;
use url::Url;

pub struct HttpClient {
    client: Client,
}

impl HttpClient {
    // Use a realistic User-Agent to avoid being blocked
    const USER_AGENT: &'static str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent(Self::USER_AGENT)
            .timeout(std::time::Duration::from_secs(30))
            .cookie_store(true)
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    pub fn get(&self, url: &Url) -> Result<String, LoadError> {
        let response = self.client
            .get(url.as_str())
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
            .send()?;
        let text = response.text()?;
        Ok(text)
    }

    pub fn get_bytes(&self, url: &Url) -> Result<Vec<u8>, LoadError> {
        log::info!("Fetching bytes from: {}", url);
        let response = self.client
            .get(url.as_str())
            .header("Accept", "image/avif,image/webp,image/apng,image/svg+xml,image/*,*/*;q=0.8")
            .send()?;

        let status = response.status();
        let final_url = response.url().to_string();
        let content_type = response.headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown");

        log::info!("Response: status={}, content-type={}, final_url={}", status, content_type, final_url);

        let bytes = response.bytes()?.to_vec();
        log::info!("Received {} bytes", bytes.len());

        Ok(bytes)
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}

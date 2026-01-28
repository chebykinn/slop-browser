use super::cache::Cache;
use super::http::HttpClient;
use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
pub enum LoadError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    #[error("Unsupported scheme: {0}")]
    UnsupportedScheme(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub struct Loader {
    http_client: HttpClient,
    cache: Cache,
}

impl Loader {
    pub fn new() -> Self {
        Self {
            http_client: HttpClient::new(),
            cache: Cache::new(),
        }
    }

    pub fn fetch(&self, url: &Url) -> Result<String, LoadError> {
        match url.scheme() {
            "http" | "https" => self.http_client.get(url),
            "file" => self.fetch_file(url),
            "data" => self.fetch_data(url),
            scheme => Err(LoadError::UnsupportedScheme(scheme.to_string())),
        }
    }

    pub fn fetch_bytes(&self, url: &Url) -> Result<Vec<u8>, LoadError> {
        match url.scheme() {
            "http" | "https" => self.http_client.get_bytes(url),
            "file" => {
                let path = url.to_file_path().map_err(|_| {
                    LoadError::InvalidUrl("Cannot convert to file path".to_string())
                })?;
                Ok(std::fs::read(path)?)
            }
            scheme => Err(LoadError::UnsupportedScheme(scheme.to_string())),
        }
    }

    fn fetch_file(&self, url: &Url) -> Result<String, LoadError> {
        let path = url.to_file_path().map_err(|_| {
            LoadError::InvalidUrl("Cannot convert to file path".to_string())
        })?;
        Ok(std::fs::read_to_string(path)?)
    }

    fn fetch_data(&self, url: &Url) -> Result<String, LoadError> {
        let data = url.path();
        if let Some(comma_pos) = data.find(',') {
            let content = &data[comma_pos + 1..];
            let header = &data[..comma_pos];

            if header.ends_with(";base64") {
                let decoded = base64_decode(content)
                    .map_err(|e| LoadError::InvalidUrl(format!("Invalid base64: {}", e)))?;
                String::from_utf8(decoded)
                    .map_err(|e| LoadError::InvalidUrl(format!("Invalid UTF-8: {}", e)))
            } else {
                Ok(urlencoding::decode(content)
                    .map_err(|e| LoadError::InvalidUrl(format!("Invalid URL encoding: {}", e)))?
                    .into_owned())
            }
        } else {
            Err(LoadError::InvalidUrl("Invalid data URL format".to_string()))
        }
    }
}

impl Default for Loader {
    fn default() -> Self {
        Self::new()
    }
}

fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    fn char_to_val(c: u8) -> Result<u8, String> {
        match c {
            b'A'..=b'Z' => Ok(c - b'A'),
            b'a'..=b'z' => Ok(c - b'a' + 26),
            b'0'..=b'9' => Ok(c - b'0' + 52),
            b'+' => Ok(62),
            b'/' => Ok(63),
            b'=' => Ok(0),
            _ => Err(format!("Invalid base64 character: {}", c as char)),
        }
    }

    let input: Vec<u8> = input.bytes().filter(|&b| b != b'\n' && b != b'\r').collect();
    let mut output = Vec::with_capacity(input.len() * 3 / 4);

    for chunk in input.chunks(4) {
        if chunk.len() < 4 {
            break;
        }

        let a = char_to_val(chunk[0])?;
        let b = char_to_val(chunk[1])?;
        let c = char_to_val(chunk[2])?;
        let d = char_to_val(chunk[3])?;

        output.push((a << 2) | (b >> 4));
        if chunk[2] != b'=' {
            output.push((b << 4) | (c >> 2));
        }
        if chunk[3] != b'=' {
            output.push((c << 6) | d);
        }
    }

    Ok(output)
}

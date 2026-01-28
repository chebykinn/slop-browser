use futures_util::StreamExt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use url::Url;

/// Token to cancel an in-progress load
#[derive(Clone)]
pub struct CancelToken {
    cancelled: Arc<AtomicBool>,
}

impl CancelToken {
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

impl Default for CancelToken {
    fn default() -> Self {
        Self::new()
    }
}

/// Progress updates from the async loader
#[derive(Debug, Clone)]
pub enum LoadProgress {
    /// Loading has started, includes content-length if known
    Started {
        url: Url,
        content_length: Option<u64>,
    },
    /// Progress update with bytes received
    Progress {
        bytes_received: u64,
        total_bytes: Option<u64>,
    },
    /// Loading completed successfully
    Complete {
        body: String,
        /// The final URL after any redirects
        final_url: Url,
    },
    /// Loading failed with an error
    Error {
        message: String,
    },
    /// Loading was cancelled
    Cancelled,
}

/// Async HTTP loader that streams content and reports progress
pub struct AsyncLoader {
    client: reqwest::Client,
}

impl AsyncLoader {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("RustBrowser/1.0")
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    /// Start loading a URL asynchronously
    /// Returns a channel receiver for progress updates
    pub fn load(
        &self,
        url: Url,
        cancel_token: CancelToken,
    ) -> mpsc::UnboundedReceiver<LoadProgress> {
        let (tx, rx) = mpsc::unbounded_channel();
        let client = self.client.clone();

        tokio::spawn(async move {
            Self::load_internal(client, url, cancel_token, tx).await;
        });

        rx
    }

    async fn load_internal(
        client: reqwest::Client,
        url: Url,
        cancel_token: CancelToken,
        tx: mpsc::UnboundedSender<LoadProgress>,
    ) {
        use std::time::Instant;
        let start = Instant::now();

        // Send started event
        let response = match client.get(url.clone()).send().await {
            Ok(resp) => resp,
            Err(e) => {
                let _ = tx.send(LoadProgress::Error {
                    message: format!("Request failed: {}", e),
                });
                return;
            }
        };
        let response_time = start.elapsed();
        println!("[Network] response in {:.0}ms for {}", response_time.as_secs_f32() * 1000.0, url);

        // Check for HTTP errors
        let status = response.status();
        if !status.is_success() {
            let _ = tx.send(LoadProgress::Error {
                message: format!("HTTP {} {}", status.as_u16(), status.canonical_reason().unwrap_or("")),
            });
            return;
        }

        let content_length = response.content_length();
        let final_url = response.url().clone();

        log::info!("Request to {} redirected to {}", url, final_url);

        let _ = tx.send(LoadProgress::Started {
            url: final_url.clone(),
            content_length,
        });

        // Stream the response body
        let mut stream = response.bytes_stream();
        let mut body = Vec::new();
        let mut bytes_received: u64 = 0;

        while let Some(chunk_result) = stream.next().await {
            // Check for cancellation
            if cancel_token.is_cancelled() {
                let _ = tx.send(LoadProgress::Cancelled);
                return;
            }

            match chunk_result {
                Ok(chunk) => {
                    bytes_received += chunk.len() as u64;
                    body.extend_from_slice(&chunk);

                    let _ = tx.send(LoadProgress::Progress {
                        bytes_received,
                        total_bytes: content_length,
                    });
                }
                Err(e) => {
                    let _ = tx.send(LoadProgress::Error {
                        message: format!("Stream error: {}", e),
                    });
                    return;
                }
            }
        }

        // Final cancellation check
        if cancel_token.is_cancelled() {
            let _ = tx.send(LoadProgress::Cancelled);
            return;
        }

        // Convert body to string
        let total_time = start.elapsed();
        println!("[Network] complete in {:.0}ms, {} bytes", total_time.as_secs_f32() * 1000.0, bytes_received);

        match String::from_utf8(body) {
            Ok(body_str) => {
                let _ = tx.send(LoadProgress::Complete { body: body_str, final_url });
            }
            Err(e) => {
                // Try lossy conversion for non-UTF8 content
                let body_str = String::from_utf8_lossy(e.as_bytes()).into_owned();
                let _ = tx.send(LoadProgress::Complete { body: body_str, final_url });
            }
        }
    }
}

impl Default for AsyncLoader {
    fn default() -> Self {
        Self::new()
    }
}

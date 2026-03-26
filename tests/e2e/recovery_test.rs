#![allow(dead_code)]
use std::sync::Arc;
use std::time::Duration;

use reqwest::Client;
use rust_kratos::infrastructure::adapters::kratos::client::KratosClient;
use serde::Deserialize;

pub struct TestContext {
    pub client: Arc<KratosClient>,
}

impl TestContext {
    pub fn new() -> Self {
        let public_url = std::env::var("KRATOS_PUBLIC_URL").unwrap_or_else(|_| "http://127.0.0.1:4433".to_string());
        let admin_url = std::env::var("KRATOS_ADMIN_URL").unwrap_or_else(|_| "http://127.0.0.1:4434".to_string());
        Self {
            client: Arc::new(KratosClient {
                client: Client::builder()
                    .cookie_store(false)
                    .redirect(reqwest::redirect::Policy::none())
                    .danger_accept_invalid_certs(true)
                    .build()
                    .expect("Failed to build HTTP client"),
                public_url,
                admin_url,
                max_retries: 3,
                retry_delay: Duration::from_millis(1000),
            }),
        }
    }

    pub fn random_email() -> String {
        format!("test_{}@example.com", uuid::Uuid::new_v4())
    }
}

pub struct MailhogClient {
    client: Client,
    base_url: String,
    kratos_public_url: String,
}

#[derive(Deserialize)]
pub struct MailhogResponse {
    pub items: Vec<MailhogMessage>,
}

#[derive(Deserialize)]
pub struct MailhogMessage {
    #[serde(rename = "Content")]
    pub content: MailhogContent,
}

#[derive(Deserialize)]
pub struct MailhogContent {
    #[serde(rename = "Body")]
    pub body: String,
}

impl MailhogClient {
    pub fn new() -> Self {
        let base_url = std::env::var("MAILHOG_API_URL").unwrap_or_else(|_| "http://127.0.0.1:8025/api/v2".to_string());
        let kratos_public_url =
            std::env::var("KRATOS_PUBLIC_URL").unwrap_or_else(|_| "http://127.0.0.1:4433".to_string());
        Self {
            client: Client::new(),
            base_url,
            kratos_public_url,
        }
    }

    pub async fn delete_all(&self) {
        let _ = self.client.delete(format!("{}/messages", self.base_url)).send().await;
    }

    pub async fn fetch_recovery_link(&self, email: &str) -> Option<String> {
        for _ in 0..10 {
            tokio::time::sleep(Duration::from_millis(500)).await;
            let resp = self
                .client
                .get(format!("{}/search?kind=to&query={}", self.base_url, email))
                .send()
                .await
                .ok()?;
            let data: MailhogResponse = resp.json().await.ok()?;
            if let Some(msg) = data.items.first() {
                let decoded = Self::decode_quoted_printable(&msg.content.body);
                if let Some(link) = Self::extract_link(&decoded, "recovery") {
                    let link = self.normalize_kratos_url(&link);
                    return Some(link);
                }
            }
        }
        None
    }

    fn decode_quoted_printable(input: &str) -> String {
        let unfolded = input.replace("=\r\n", "").replace("=\n", "");
        let mut result = String::with_capacity(unfolded.len());
        let mut chars = unfolded.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '=' {
                let a = chars.next();
                let b = chars.next();
                match (a, b) {
                    (Some(x), Some(y)) => {
                        let hex = format!("{}{}", x, y);
                        if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                            result.push(byte as char);
                        } else {
                            result.push('=');
                            result.push(x);
                            result.push(y);
                        }
                    }
                    (Some(x), None) => {
                        result.push('=');
                        result.push(x);
                    }
                    _ => result.push('='),
                }
            } else {
                result.push(c);
            }
        }
        result
    }

    fn normalize_kratos_url(&self, link: &str) -> String {
        let internal_hosts = ["http://kratos:4433", "https://kratos:4433"];
        let mut result = link.to_string();
        for host in &internal_hosts {
            if result.starts_with(host) {
                result = result.replacen(host, &self.kratos_public_url, 1);
                break;
            }
        }
        result
    }

    fn extract_link(body: &str, contains: &str) -> Option<String> {
        let start = body.find("http")?;
        let link: String = body[start..]
            .chars()
            .take_while(|c| !c.is_whitespace() && *c != '"' && *c != '<')
            .collect();
        if link.contains(contains) { Some(link) } else { None }
    }
}

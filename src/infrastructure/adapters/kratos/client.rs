use crate::application::bootstrap::config::KratosConfig;
use reqwest::Client;
use std::time::Duration;

#[derive(Clone)]
pub struct KratosClient {
    pub client: Client,
    pub admin_url: String,
    pub public_url: String,
    pub max_retries: u32,
    pub retry_delay: Duration,
}

impl KratosClient {
    pub fn new(config: &KratosConfig) -> Self {
        let client = Client::builder()
            .cookie_store(false)
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(config.timeout_secs))
            .connect_timeout(Duration::from_secs(config.connect_timeout_secs))
            .pool_idle_timeout(Duration::from_secs(config.pool_idle_timeout_secs))
            .pool_max_idle_per_host(config.pool_max_idle_per_host)
            .danger_accept_invalid_certs(config.accept_invalid_certs)
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client,
            admin_url: config.admin_url.clone(),
            public_url: config.public_url.clone(),
            max_retries: config.max_retries,
            retry_delay: Duration::from_millis(config.retry_delay_ms),
        }
    }

    pub async fn wait_until_ready(&self) -> Result<(), reqwest::Error> {
        let url = format!("{}/health/ready", self.public_url);
        let mut last_err: Option<reqwest::Error> = None;

        for attempt in 1..=self.max_retries {
            match self.client.get(&url).send().await {
                Ok(resp) if resp.status().is_success() => {
                    tracing::info!(attempt, "Kratos is ready");
                    return Ok(());
                }
                Ok(resp) => {
                    tracing::warn!(
                        attempt,
                        max = self.max_retries,
                        status = %resp.status(),
                        "Kratos not ready, retrying in {:?}", self.retry_delay
                    );
                }
                Err(e) => {
                    tracing::warn!(
                        attempt,
                        max = self.max_retries,
                        error = %e,
                        "Kratos unreachable, retrying in {:?}", self.retry_delay
                    );
                    last_err = Some(e);
                }
            }

            if attempt < self.max_retries {
                tokio::time::sleep(self.retry_delay).await;
            }
        }

        Err(last_err.expect("no error captured but Kratos never became ready"))
    }

    pub async fn execute_with_retry<F, Fut, T, E>(&self, mut operation: F) -> Result<T, E>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Display,
    {
        let mut attempts = 0;

        loop {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    attempts += 1;

                    if attempts >= self.max_retries {
                        return Err(e);
                    }

                    tracing::warn!(
                        attempt = attempts,
                        max = self.max_retries,
                        error = %e,
                        "Request failed, retrying in {:?}", self.retry_delay
                    );

                    tokio::time::sleep(self.retry_delay).await;
                }
            }
        }
    }
}

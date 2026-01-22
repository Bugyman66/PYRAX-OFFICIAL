use crate::config::TelegramConfig;
use tracing::{info, error, warn};
use reqwest::Client;
use std::time::{Duration, Instant};
use std::sync::Mutex;

pub struct AlertManager {
    config: TelegramConfig,
    client: Client,
    last_alerts: Mutex<std::collections::HashMap<String, Instant>>,
}

impl AlertManager {
    pub fn new(config: TelegramConfig) -> Self {
        Self {
            config,
            client: Client::new(),
            last_alerts: Mutex::new(std::collections::HashMap::new()),
        }
    }

    /// Send a critical alert (Stall, Fork)
    pub async fn send_alert(&self, title: &str, message: &str) {
        if !self.config.enabled || self.config.bot_token.is_none() || self.config.chat_id.is_none() {
            return;
        }

        // De-bounce: Don't send the same alert type more than once every 5 minutes
        // We use 'title' as the key
        {
            let mut last = self.last_alerts.lock().unwrap();
            if let Some(time) = last.get(title) {
                if time.elapsed() < Duration::from_secs(300) {
                    warn!("Suppressed duplicate alert: {}", title);
                    return;
                }
            }
            last.insert(title.to_string(), Instant::now());
        }

        let token = self.config.bot_token.as_ref().unwrap();
        let chat_id = self.config.chat_id.as_ref().unwrap();
        
        let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
        let text = format!("<b>{}</b>\n\n{}", title, message);

        let params = [
            ("chat_id", chat_id),
            ("text", &text),
            ("parse_mode", &"HTML".to_string()),
        ];

        match self.client.post(&url).form(&params).send().await {
            Ok(res) => {
                if !res.status().is_success() {
                    let err_text = res.text().await.unwrap_or_default();
                    error!("Failed to send Telegram alert: {}", err_text);
                } else {
                    info!("Sent Telegram alert: {}", title);
                }
            }
            Err(e) => {
                error!("Failed to send Telegram request: {}", e);
            }
        }
    }
}

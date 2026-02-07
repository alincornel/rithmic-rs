//! Example: Persistent connection with automatic reconnection and re-subscription
//!
//! Uses `ConnectStrategy::Retry` so the initial connection retries automatically
//! with exponential backoff. The outer loop handles reconnection after disconnects.
//!
//! Run with: cargo run --example reconnect
//!
//! This example runs indefinitely. Press Ctrl+C to exit.

use std::{collections::HashSet, env};
use tracing::{error, info, warn};

use rithmic_rs::{
    ConnectStrategy, RithmicConfig, RithmicEnv, RithmicTickerPlant, rti::messages::RithmicMessage,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt().init();

    let config = RithmicConfig::from_env(RithmicEnv::Demo)?;

    // Track subscriptions to restore after reconnect
    let mut subscriptions: HashSet<(String, String)> = HashSet::new();
    let symbol = env::var("SYMBOL").unwrap_or_else(|_| "ESH6".to_string());
    let exchange = env::var("EXCHANGE").unwrap_or_else(|_| "CME".to_string());
    subscriptions.insert((symbol, exchange));

    // Outer loop: reconnection
    loop {
        // Retry strategy handles backoff automatically
        let plant = match RithmicTickerPlant::connect(&config, ConnectStrategy::Retry).await {
            Ok(p) => p,
            Err(e) => {
                error!("Connect failed: {}", e);
                continue;
            }
        };

        let mut handle = plant.get_handle();

        if let Err(e) = handle.login().await {
            error!("Login failed: {}", e);
            continue;
        }

        // Subscribe to all tracked symbols
        for (symbol, exchange) in &subscriptions {
            let _ = handle.subscribe(symbol, exchange).await;
            info!("Subscribed to {} on {}", symbol, exchange);
        }

        // Process until disconnect
        while let Ok(update) = handle.subscription_receiver.recv().await {
            match &update.message {
                RithmicMessage::HeartbeatTimeout
                | RithmicMessage::ForcedLogout(_)
                | RithmicMessage::ConnectionError => {
                    warn!("Disconnected, reconnecting...");
                    break;
                }
                RithmicMessage::LastTrade(t) => {
                    info!(
                        "Trade: {} @ {}",
                        t.trade_size.unwrap_or(0),
                        t.trade_price.unwrap_or(0.0)
                    );
                }
                _ => {}
            }
        }
    }
}

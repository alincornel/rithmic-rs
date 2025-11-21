//! Test: Heartbeat request and response logging
//!
//! This test binary verifies that heartbeat requests and responses work correctly.
//! It enables heartbeat response monitoring and logs all heartbeat-related activity.

use std::time::Duration;
use tracing::{info, warn, error};

use rithmic_rs::{
    ConnectStrategy, RithmicConfig, RithmicEnv, RithmicTickerPlant,
    rti::messages::RithmicMessage,
    ws::RithmicStream,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("=== Heartbeat Test Starting ===");

    // Load configuration
    let config = RithmicConfig::from_dotenv(RithmicEnv::Demo)?;

    // Connect to ticker plant
    info!("Connecting to ticker plant...");
    let ticker_plant = RithmicTickerPlant::connect(&config, ConnectStrategy::Simple).await?;
    let mut handle = ticker_plant.get_handle();

    // Login
    info!("Logging in...");
    handle.login().await?;
    info!("Login successful");

    // Enable heartbeat response monitoring
    info!("Enabling heartbeat response monitoring...");
    handle.return_heartbeat_response(true).await;
    info!("Heartbeat monitoring enabled - will receive HeartbeatTimeout on failures");

    // Monitor for heartbeat activity
    info!("Monitoring heartbeat activity for 180 seconds (expecting 3 heartbeats at 60s intervals)...");
    info!("NOTE: Successful heartbeats are SILENT - only timeouts are reported");

    let start = tokio::time::Instant::now();
    let test_duration = Duration::from_secs(180);

    let mut heartbeat_count = 0;
    let mut timeout_count = 0;

    loop {
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_secs(1)) => {
                let elapsed = start.elapsed();
                if elapsed >= test_duration {
                    info!("Test duration reached ({}s)", elapsed.as_secs());
                    break;
                }

                // Log progress every 30 seconds
                if elapsed.as_secs() % 30 == 0 && elapsed.as_secs() > 0 {
                    info!("Test running: {}s elapsed, {} timeouts detected",
                          elapsed.as_secs(), timeout_count);
                }
            }

            result = handle.subscription_receiver.recv() => {
                match result {
                    Ok(response) => {
                        match response.message {
                            RithmicMessage::HeartbeatTimeout => {
                                timeout_count += 1;
                                warn!("❌ HEARTBEAT TIMEOUT detected!");
                                warn!("   Request ID: {}", response.request_id);
                                warn!("   Error: {}", response.error.unwrap_or_else(|| "Unknown".to_string()));
                                warn!("   This indicates the server did not respond within 30 seconds");
                            }
                            RithmicMessage::ResponseHeartbeat(_) => {
                                // This should NOT happen - heartbeats should be filtered
                                heartbeat_count += 1;
                                error!("⚠️  UNEXPECTED: Received ResponseHeartbeat in subscription channel!");
                                error!("   This should not happen - successful heartbeats should be silent");
                                error!("   Request ID: {}", response.request_id);
                            }
                            RithmicMessage::ConnectionError => {
                                error!("❌ CONNECTION ERROR");
                                error!("   Error: {}", response.error.unwrap_or_else(|| "Unknown".to_string()));
                                break;
                            }
                            RithmicMessage::ForcedLogout(_) => {
                                warn!("Server forced logout");
                                break;
                            }
                            _ => {
                                // Ignore other messages
                            }
                        }
                    }
                    Err(e) => {
                        error!("Channel error: {}", e);
                        break;
                    }
                }
            }
        }
    }

    let elapsed = start.elapsed();

    info!("=== Test Complete ===");
    info!("Duration: {:.1}s", elapsed.as_secs_f64());
    info!("Expected heartbeats: ~{}", elapsed.as_secs() / 60);
    info!("Heartbeat timeouts: {}", timeout_count);
    info!("Unexpected ResponseHeartbeat messages: {}", heartbeat_count);

    if heartbeat_count > 0 {
        error!("❌ TEST FAILED: Received {} unexpected ResponseHeartbeat messages", heartbeat_count);
        error!("   Successful heartbeats should be filtered and NOT appear in subscription channel");
    } else if timeout_count > 0 {
        warn!("⚠️  TEST WARNING: {} heartbeat timeouts detected", timeout_count);
        warn!("   This may indicate network issues or server delays");
    } else {
        info!("✅ TEST PASSED: No timeouts, no unexpected heartbeat messages");
        info!("   Heartbeat system working correctly (silent success, noisy failure)");
    }

    // Disconnect
    info!("Disconnecting...");
    handle.disconnect().await?;
    info!("Disconnected");

    Ok(())
}

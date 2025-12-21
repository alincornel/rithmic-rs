# Rust Rithmic R | Protocol API client

[![Crates.io](https://img.shields.io/crates/v/rithmic-rs.svg)](https://crates.io/crates/rithmic-rs)
[![Documentation](https://docs.rs/rithmic-rs/badge.svg)](https://docs.rs/rithmic-rs)

[Official Rithmic API](https://www.rithmic.com/apis)

Unofficial rust client for connecting to Rithmic's R | Protocol API.

## Setup

You can install it from crates.io

```
$ cargo add rithmic-rs
```

Or manually add it to your `Cargo.toml` file.

```
[dependencies]
rithmic-rs = "0.6.2"
```

## Usage

### Configuration

Rithmic supports three types of account environments: `RithmicEnv::Demo` for paper trading, `RithmicEnv::Live` for funded accounts, and `RithmicEnv::Test` for the test environment before app approval.

There are two ways to configure your connection: loading from environment variables (recommended) or using the builder pattern programmatically.

Load configuration from environment variables:

```rust
use rithmic_rs::{RithmicConfig, RithmicEnv};

let config = RithmicConfig::from_env(RithmicEnv::Demo)?;
```

Required environment variables:
```sh
# For Demo environment
RITHMIC_DEMO_ACCOUNT_ID=your_account_id
RITHMIC_DEMO_FCM_ID=your_fcm_id
RITHMIC_DEMO_IB_ID=your_ib_id
RITHMIC_DEMO_USER=your_username
RITHMIC_DEMO_PW=your_password
RITHMIC_DEMO_URL=<provided_by_rithmic>
RITHMIC_DEMO_ALT_URL=<provided_by_rithmic>

# For Live environment
RITHMIC_LIVE_ACCOUNT_ID=your_account_id
RITHMIC_LIVE_FCM_ID=your_fcm_id
RITHMIC_LIVE_IB_ID=your_ib_id
RITHMIC_LIVE_USER=your_username
RITHMIC_LIVE_PW=your_password
RITHMIC_LIVE_URL=<provided_by_rithmic>
RITHMIC_LIVE_ALT_URL=<provided_by_rithmic>

# For Test environment
RITHMIC_TEST_ACCOUNT_ID=your_account_id
RITHMIC_TEST_FCM_ID=your_fcm_id
RITHMIC_TEST_IB_ID=your_ib_id
RITHMIC_TEST_USER=your_username
RITHMIC_TEST_PW=your_password
RITHMIC_TEST_URL=<provided_by_rithmic>
RITHMIC_TEST_ALT_URL=<provided_by_rithmic>
```

See `examples/.env.blank` for a template with all required variables and connection URLs.

Alternatively, configure using the builder pattern:

```rust
use rithmic_rs::{RithmicConfig, RithmicEnv};

let config = RithmicConfig::builder()
    .user("your_username".to_string())
    .password("your_password".to_string())
    .system_name("Rithmic Paper Trading".to_string())
    .env(RithmicEnv::Demo)
    .build()?;
```

### Connection Strategies

The library provides three connection strategies for **initial connection**:
- **`Simple`**: Single connection attempt (recommended default, fast-fail)
- **`Retry`**: Indefinite retries with exponential backoff capped at 60 seconds
- **`AlternateWithRetry`**: Alternates between primary and beta URLs with retries

**Note**: These strategies are for establishing the initial connection only. If you need to reconnect after a connection is lost, you must listen for disconnection events and manually reconnect:

```rust
use rithmic_rs::{RithmicMessage, RithmicConfig, RithmicEnv, ConnectStrategy, RithmicTickerPlant};

async fn maintain_connection() -> Result<(), Box<dyn std::error::Error>> {
    let config = RithmicConfig::from_env(RithmicEnv::Demo)?;

    loop {
        // Connect and get handle
        let ticker_plant = RithmicTickerPlant::connect(&config, ConnectStrategy::Retry).await?;
        let handle = ticker_plant.get_handle();

        handle.login().await?;
        handle.subscribe("ESU5", "CME").await?;

        // Process messages until disconnection
        loop {
            match handle.subscription_receiver.recv().await {
                Ok(update) => {
                    // Check for disconnection events
                    match update.message {
                        RithmicMessage::HeartbeatTimeout |
                        RithmicMessage::ForcedLogout(_) |
                        RithmicMessage::ConnectionError => {
                            eprintln!("Connection lost, reconnecting...");
                            break; // Exit inner loop to reconnect
                        }
                        RithmicMessage::LastTrade(trade) => {
                            println!("Trade: {} @ {}", trade.trade_size.unwrap(), trade.trade_price.unwrap());
                        }
                        _ => {}
                    }
                }
                Err(_) => break, // Channel closed, reconnect
            }
        }

        // Brief delay before reconnecting
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}
```

### Quick Start

To use this crate, create a configuration and connect to a plant with your chosen strategy. Each plant uses the actor pattern and spawns a task that listens to commands via a handle. Plants like the ticker plant also include a broadcast channel for real-time updates.

```rust
use rithmic_rs::{RithmicConfig, RithmicEnv, ConnectStrategy, RithmicTickerPlant};

async fn stream_live_ticks() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration from environment variables
    let config = RithmicConfig::from_env(RithmicEnv::Demo)?;

    // Connect with retry strategy (indefinite retries with 60s max backoff)
    let ticker_plant = RithmicTickerPlant::connect(&config, ConnectStrategy::Retry).await?;
    let handle = ticker_plant.get_handle();

    // Login and subscribe
    handle.login().await?;
    handle.subscribe("ESU5", "CME").await?;

    // Process real-time updates
    loop {
        match handle.subscription_receiver.recv().await {
            Ok(update) => {
                // Check for errors on all messages
                if let Some(error) = &update.error {
                    eprintln!("Error from {}: {}", update.source, error);
                }

                // Handle connection health issues
                // See Connection Strategies section for reconnection example
                match update.message {
                    RithmicMessage::HeartbeatTimeout => {
                        eprintln!("Connection timeout detected");
                        break;
                    }
                    RithmicMessage::ForcedLogout(_) => {
                        eprintln!("Forced logout detected");
                        break;
                    }
                    RithmicMessage::ConnectionError => {
                        eprintln!("Connection error detected");
                        break;
                    }
                    _ => {}
                }

                // Process market data
                match update.message {
                    RithmicMessage::LastTrade(trade) => {
                        println!("Trade: {} @ {}", trade.trade_size.unwrap(), trade.trade_price.unwrap());
                    }
                    RithmicMessage::BestBidOffer(bbo) => {
                        println!("BBO: {}@{} / {}@{}",
                            bbo.bid_size.unwrap(), bbo.bid_price.unwrap(),
                            bbo.ask_price.unwrap(), bbo.ask_size.unwrap());
                    }
                    _ => {}
                }
            }
            Err(e) => {
                eprintln!("Channel error: {}", e);
                break;
            }
        }
    }

    Ok(())
}
```

## Plants

Rithmic's API is organized into specialized services called "plants". Each plant uses the actor pattern - you connect to a plant and communicate with it via a handle using tokio channels.

### Design Philosophy

This library intentionally does **not** provide a single unified client that manages all plants. Instead, each plant is an independent actor that you connect to and manage separately. This design choice prioritizes **flexibility** over convenience:

- **Thread/task distribution**: Run different plants on different threads or async tasks based on your performance needs
- **Selective connectivity**: Connect only to the plants you need (e.g., market data only, no order management)
- **Independent lifecycle**: Each plant can be started, stopped, and reconnected independently
- **Resource control**: Fine-grained control over connection pooling and resource allocation

The trade-off is **reduced usability** - you must manage multiple connections and handles yourself rather than having a single client object. For most applications, this flexibility is worth the additional setup code.

### Ticker Plant

Real-time market data streaming:
- **Market data subscriptions**: Last trades, best bid/offer (BBO), order book depth
- **Symbol discovery**: Search symbols, list exchanges, get instruments by underlying
- **Reference data**: Tick size tables, product codes, front month contracts, volume at price

### Order Plant

Order management and execution:
- **Order types**: Market, limit, bracket orders, OCO (one-cancels-other) orders
- **Order operations**: Place, modify, cancel orders, exit positions
- **Order tracking**: Subscribe to order updates, show active orders, order history
- **Risk management**: Account/product RMS info, trade routes, easy-to-borrow lists
- **Agreements**: List, accept, and manage exchange agreements

### PnL Plant

Profit and loss monitoring:
- **Position snapshots**: Current P&L for all positions
- **Real-time updates**: Subscribe to account and instrument-level P&L changes

### History Plant

Historical market data:
- **Tick data**: Load historical tick-by-tick data for any time range
- **Time bars**: 1-second, 1-minute, 5-minute, daily, and weekly bars
- **Volume profile**: Minute bars with volume profile data
- **Bar subscriptions**: Subscribe to real-time time bar and tick bar updates


## Examples

The repository includes several examples to help you get started. Examples use environment variables for configuration - set the required variables (listed above) in your environment or use a `.env` file.

### Basic Connection
```bash
cargo run --example connect
```

### Market Data
```bash
# Stream live market data
cargo run --example market_data
```

### Historical Data
```bash
# Load historical tick data
cargo run --example load_historical_ticks

# Load historical time bars (1-minute, 5-minute, daily)
cargo run --example load_historical_bars
```

## Upgrading

See [CHANGELOG.md](CHANGELOG.md) for version history and [MIGRATION_0.6.0.md](MIGRATION_0.6.0.md) for migration guides.

## Contribution

Contributions encouraged and welcomed!

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as below, without any additional terms or conditions.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.

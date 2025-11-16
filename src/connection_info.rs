//! Deprecated configuration types for backward compatibility.
//!
//! **DEPRECATED**: This module is maintained for backward compatibility only.
//! New code should use the [`crate::config`] module instead.
//!
//! # Migration Guide
//!
//! Old code:
//! ```ignore
//! use rithmic_rs::connection_info::{AccountInfo, RithmicConnectionSystem, get_config};
//!
//! let account_info = AccountInfo {
//!     account_id: "my_account".to_string(),
//!     env: RithmicConnectionSystem::Demo,
//!     fcm_id: "my_fcm".to_string(),
//!     ib_id: "my_ib".to_string(),
//! };
//! let conn_info = get_config(&account_info.env);
//! ```
//!
//! New code:
//! ```ignore
//! use rithmic_rs::config::{RithmicConfig, RithmicEnv};
//!
//! let config = RithmicConfig::from_env(RithmicEnv::Demo)?;
//! ```

use dotenv::dotenv;
use std::{env, fmt, str::FromStr};

/// Deprecated: Connection information structure.
///
/// **DEPRECATED**: Use [`crate::config::RithmicConfig`] instead.
///
/// This type is maintained for backward compatibility. It contains only
/// connection-related fields (URL, credentials, system name).
#[deprecated(
    since = "0.5.0",
    note = "Use `config::RithmicConfig` instead for a unified configuration type"
)]
#[derive(Clone, Debug)]
pub struct RithmicConnectionInfo {
    pub url: String,
    pub beta_url: String,
    pub user: String,
    pub password: String,
    pub system_name: String,
}

/// Deprecated: Trading environment selector.
///
/// **DEPRECATED**: Use [`crate::config::RithmicEnv`] instead.
#[deprecated(since = "0.5.0", note = "Use `config::RithmicEnv` instead")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RithmicConnectionSystem {
    Demo,
    Live,
    Test,
}

impl fmt::Display for RithmicConnectionSystem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RithmicConnectionSystem::Demo => write!(f, "demo"),
            RithmicConnectionSystem::Live => write!(f, "live"),
            RithmicConnectionSystem::Test => write!(f, "test"),
        }
    }
}

impl FromStr for RithmicConnectionSystem {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "demo" | "development" => Ok(RithmicConnectionSystem::Demo),
            "live" | "production" => Ok(RithmicConnectionSystem::Live),
            "test" => Ok(RithmicConnectionSystem::Test),
            _ => Err(()),
        }
    }
}

/// Deprecated: Get connection configuration for an environment.
///
/// **DEPRECATED**: Use [`crate::config::RithmicConfig::from_env`] instead.
///
/// This function loads connection information from environment variables
/// and returns a `RithmicConnectionInfo` struct. It calls `dotenv()` internally.
///
/// # Panics
/// Panics if required environment variables are not set.
#[deprecated(
    since = "0.5.0",
    note = "Use `config::RithmicConfig::from_env()` or `config::RithmicConfig::from_dotenv()` instead"
)]
pub fn get_config(env: &RithmicConnectionSystem) -> RithmicConnectionInfo {
    dotenv().ok();

    match env {
        RithmicConnectionSystem::Demo => RithmicConnectionInfo {
            url: "wss://rprotocol.rithmic.com:443".into(),
            beta_url: "wss://rprotocol-beta.rithmic.com:443".into(),
            user: env::var("RITHMIC_DEMO_USER").unwrap(),
            password: env::var("RITHMIC_DEMO_PW").unwrap(),
            system_name: "Rithmic Paper Trading".into(),
        },
        RithmicConnectionSystem::Live => RithmicConnectionInfo {
            url: "wss://rprotocol.rithmic.com:443".into(),
            beta_url: "wss://rprotocol-beta.rithmic.com:443".into(),
            user: env::var("RITHMIC_LIVE_USER").unwrap(),
            password: env::var("RITHMIC_LIVE_PW").unwrap(),
            system_name: "Rithmic 01".into(),
        },
        RithmicConnectionSystem::Test => RithmicConnectionInfo {
            url: "wss://rituz00100.rithmic.com:443".into(),
            beta_url: "wss://rprotocol-beta.rithmic.com:443".into(),
            user: env::var("RITHMIC_TEST_USER").unwrap(),
            password: env::var("RITHMIC_TEST_PW").unwrap(),
            system_name: "Rithmic Test".into(),
        },
    }
}

/// Deprecated: Account information structure.
///
/// **DEPRECATED**: Use [`crate::config::RithmicConfig`] instead.
///
/// This type is maintained for backward compatibility. The new `RithmicConfig`
/// type consolidates both account and connection information.
#[deprecated(
    since = "0.5.0",
    note = "Use `config::RithmicConfig` instead for a unified configuration type"
)]
#[derive(Clone, Debug)]
pub struct AccountInfo {
    pub account_id: String,
    pub env: RithmicConnectionSystem,
    pub fcm_id: String,
    pub ib_id: String,
}

// ============================================================================
// Conversion implementations between old and new types
// ============================================================================

use crate::config::{ConfigError, RithmicConfig, RithmicEnv};

/// Convert from old RithmicConnectionSystem to new RithmicEnv
impl From<RithmicConnectionSystem> for RithmicEnv {
    fn from(old: RithmicConnectionSystem) -> Self {
        match old {
            RithmicConnectionSystem::Demo => RithmicEnv::Demo,
            RithmicConnectionSystem::Live => RithmicEnv::Live,
            RithmicConnectionSystem::Test => RithmicEnv::Test,
        }
    }
}

/// Convert from new RithmicEnv to old RithmicConnectionSystem
#[allow(deprecated)]
impl From<RithmicEnv> for RithmicConnectionSystem {
    fn from(new: RithmicEnv) -> Self {
        match new {
            RithmicEnv::Demo => RithmicConnectionSystem::Demo,
            RithmicEnv::Live => RithmicConnectionSystem::Live,
            RithmicEnv::Test => RithmicConnectionSystem::Test,
        }
    }
}

/// Convert from old AccountInfo to new RithmicConfig
///
/// This attempts to load connection info from environment variables.
/// If you need more control, use `RithmicConfig::builder()` instead.
#[allow(deprecated)]
impl TryFrom<AccountInfo> for RithmicConfig {
    type Error = ConfigError;

    fn try_from(old: AccountInfo) -> Result<Self, Self::Error> {
        dotenv().ok();

        let env: RithmicEnv = old.env.into();

        // Load connection info from environment
        let (url, beta_url, user, password, system_name) = match &env {
            RithmicEnv::Demo => (
                "wss://rprotocol.rithmic.com:443".to_string(),
                "wss://rprotocol-beta.rithmic.com:443".to_string(),
                env::var("RITHMIC_DEMO_USER")
                    .map_err(|_| ConfigError::MissingEnvVar("RITHMIC_DEMO_USER".to_string()))?,
                env::var("RITHMIC_DEMO_PW")
                    .map_err(|_| ConfigError::MissingEnvVar("RITHMIC_DEMO_PW".to_string()))?,
                "Rithmic Paper Trading".to_string(),
            ),
            RithmicEnv::Live => (
                "wss://rprotocol.rithmic.com:443".to_string(),
                "wss://rprotocol-beta.rithmic.com:443".to_string(),
                env::var("RITHMIC_LIVE_USER")
                    .map_err(|_| ConfigError::MissingEnvVar("RITHMIC_LIVE_USER".to_string()))?,
                env::var("RITHMIC_LIVE_PW")
                    .map_err(|_| ConfigError::MissingEnvVar("RITHMIC_LIVE_PW".to_string()))?,
                "Rithmic 01".to_string(),
            ),
            RithmicEnv::Test => (
                "wss://rituz00100.rithmic.com:443".to_string(),
                "wss://rprotocol-beta.rithmic.com:443".to_string(),
                env::var("RITHMIC_TEST_USER")
                    .map_err(|_| ConfigError::MissingEnvVar("RITHMIC_TEST_USER".to_string()))?,
                env::var("RITHMIC_TEST_PW")
                    .map_err(|_| ConfigError::MissingEnvVar("RITHMIC_TEST_PW".to_string()))?,
                "Rithmic Test".to_string(),
            ),
        };

        Ok(RithmicConfig {
            account_id: old.account_id,
            fcm_id: old.fcm_id,
            ib_id: old.ib_id,
            url,
            beta_url,
            user,
            password,
            system_name,
            env,
        })
    }
}

/// Convert from new RithmicConfig to old AccountInfo
///
/// This extracts only the account-related fields. Connection information is lost.
#[allow(deprecated)]
impl From<RithmicConfig> for AccountInfo {
    fn from(new: RithmicConfig) -> Self {
        AccountInfo {
            account_id: new.account_id,
            env: new.env.into(),
            fcm_id: new.fcm_id,
            ib_id: new.ib_id,
        }
    }
}

/// Convert from new RithmicConfig to old RithmicConnectionInfo
///
/// This extracts only the connection-related fields. Account information is lost.
#[allow(deprecated)]
impl From<RithmicConfig> for RithmicConnectionInfo {
    fn from(new: RithmicConfig) -> Self {
        RithmicConnectionInfo {
            url: new.url,
            beta_url: new.beta_url,
            user: new.user,
            password: new.password,
            system_name: new.system_name,
        }
    }
}

/// Convert from RithmicConfig reference to old AccountInfo
#[allow(deprecated)]
impl From<&RithmicConfig> for AccountInfo {
    fn from(new: &RithmicConfig) -> Self {
        AccountInfo {
            account_id: new.account_id.clone(),
            env: new.env.clone().into(),
            fcm_id: new.fcm_id.clone(),
            ib_id: new.ib_id.clone(),
        }
    }
}

/// Convert from RithmicConfig reference to old RithmicConnectionInfo
#[allow(deprecated)]
impl From<&RithmicConfig> for RithmicConnectionInfo {
    fn from(new: &RithmicConfig) -> Self {
        RithmicConnectionInfo {
            url: new.url.clone(),
            beta_url: new.beta_url.clone(),
            user: new.user.clone(),
            password: new.password.clone(),
            system_name: new.system_name.clone(),
        }
    }
}

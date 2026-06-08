//! Validating a service config from raw, untrusted strings — reporting every
//! problem at once, with the credential kept out of logs. All through one name.
//!
//! Three building blocks cooperate, reached through the single `reliakit` crate:
//!
//! - [`reliakit::primitives`] gives typed fields that reject bad input when built.
//! - [`reliakit::validate`] collects every error instead of failing on the first.
//! - [`reliakit::secret`] keeps the API key out of `Debug`/`Display`/logs.
//!
//! Run it:
//!
//! ```sh
//! cargo run -p reliakit --example config_check \
//!   --features "primitives validate secret"
//! ```

use reliakit::primitives::{BoundedStr, Email, Port};
use reliakit::secret::{ExposeSecret, SecretString};
use reliakit::validate::{ValidationError, Violation};

/// A service name is 3–32 characters.
type ServiceName = BoundedStr<3, 32>;

/// Raw, untrusted config as it might arrive from environment variables or a file.
struct RawConfig {
    name: &'static str,
    port: &'static str,
    admin_email: &'static str,
    api_key: &'static str,
}

/// The validated config. Once one exists, every field already holds its invariant.
struct ServiceConfig {
    name: ServiceName,
    port: Port,
    admin_email: Email,
    api_key: SecretString,
}

impl ServiceConfig {
    /// Parse every field, collecting a violation per problem so the caller sees
    /// the whole list at once instead of fixing errors one reload at a time.
    fn parse(raw: &RawConfig) -> Result<ServiceConfig, ValidationError> {
        let mut errors = ValidationError::empty();

        let name = ServiceName::new(raw.name).ok();
        if name.is_none() {
            errors.push(Violation::with_field("name", "must be 3-32 characters"));
        }

        let port = raw.port.parse::<u16>().ok().and_then(|p| Port::new(p).ok());
        if port.is_none() {
            errors.push(Violation::with_field("port", "must be a number in 1-65535"));
        }

        let admin_email = Email::new(raw.admin_email).ok();
        if admin_email.is_none() {
            errors.push(Violation::with_field(
                "admin_email",
                "must be a valid email",
            ));
        }

        // The secret stays wrapped even while a policy is checked on it.
        let api_key = SecretString::from_string(raw.api_key);
        if api_key.expose_secret().len() < 8 {
            errors.push(Violation::with_field(
                "api_key",
                "must be at least 8 characters",
            ));
        }

        match (name, port, admin_email) {
            (Some(name), Some(port), Some(admin_email)) if errors.is_empty() => Ok(ServiceConfig {
                name,
                port,
                admin_email,
                api_key,
            }),
            _ => Err(errors),
        }
    }
}

fn main() {
    // A config with four bad fields. Every problem is reported in one pass.
    let bad = RawConfig {
        name: "x",
        port: "99999",
        admin_email: "not-an-email",
        api_key: "short",
    };

    println!("checking a bad config:");
    match ServiceConfig::parse(&bad) {
        Ok(_) => println!("  ok"),
        Err(errors) => {
            for v in errors.violations() {
                println!("  - {}: {}", v.field.unwrap_or("(config)"), v.message);
            }
        }
    }

    // A valid config. The secret never appears in output.
    let good = RawConfig {
        name: "api-service",
        port: "8080",
        admin_email: "admin@example.com",
        api_key: "rk_live_secret_value",
    };

    println!("\nchecking a good config:");
    match ServiceConfig::parse(&good) {
        Ok(config) => {
            println!("  service '{}' on port {}", config.name, config.port);
            println!("  admin: {}", config.admin_email);
            println!("  api key (display): {}", config.api_key); // -> [REDACTED]
            println!("  api key length: {}", config.api_key.expose_secret().len());
        }
        Err(_) => println!("  unexpected validation error"),
    }
}

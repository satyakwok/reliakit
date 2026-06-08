//! Collect every validation error at once instead of failing on the first.
//!
//! ```sh
//! cargo run -p reliakit-validate --example basic
//! ```

use reliakit_validate::{Validate, ValidationError, Violation};

/// A signup form validated as a whole, so the user sees every problem in one go.
struct Signup {
    username: String,
    age: u32,
    email: String,
}

impl Validate for Signup {
    type Error = ValidationError;

    fn validate(&self) -> Result<(), Self::Error> {
        let mut errors = ValidationError::empty();

        if self.username.len() < 3 {
            errors.push(Violation::with_field(
                "username",
                "must be at least 3 characters",
            ));
        }
        if self.age < 18 {
            errors.push(Violation::with_field("age", "must be 18 or older"));
        }
        if !self.email.contains('@') {
            errors.push(Violation::with_field("email", "must contain '@'"));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

fn main() {
    let bad = Signup {
        username: "jo".into(),
        age: 15,
        email: "nope".into(),
    };

    match bad.validate() {
        Ok(()) => println!("valid"),
        Err(errors) => {
            println!("{} problem(s):", errors.len());
            for v in errors.violations() {
                println!("  - {}: {}", v.field.unwrap_or("(form)"), v.message);
            }
        }
    }

    let good = Signup {
        username: "jordan".into(),
        age: 30,
        email: "jordan@example.com".into(),
    };
    println!("good signup valid: {}", good.validate().is_ok());
}

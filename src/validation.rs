//! Input validation for user-provided data.

/// Validation result with error message.
pub type ValidationResult = Result<(), String>;

/// Validates a secret name according to GCP Secret Manager rules.
///
/// Rules:
/// - Must be 1-255 characters
/// - Must start with a letter
/// - Can contain letters, digits, underscores, and hyphens
/// - Cannot end with a hyphen
pub fn validate_secret_name(name: &str) -> ValidationResult {
    // Check empty
    if name.is_empty() {
        return Err("Secret name cannot be empty".to_string());
    }

    // Check length
    if name.len() > 255 {
        return Err("Secret name must be 255 characters or less".to_string());
    }

    // Check first character is a letter
    let first = name.chars().next().unwrap();
    if !first.is_ascii_alphabetic() {
        return Err("Secret name must start with a letter".to_string());
    }

    // Check all characters are valid
    for c in name.chars() {
        if !c.is_ascii_alphanumeric() && c != '_' && c != '-' {
            return Err(format!(
                "Secret name can only contain letters, digits, underscores, and hyphens. Found: '{c}'"
            ));
        }
    }

    // Check doesn't end with hyphen
    if name.ends_with('-') {
        return Err("Secret name cannot end with a hyphen".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_secret_names() {
        assert!(validate_secret_name("my-secret").is_ok());
        assert!(validate_secret_name("my_secret").is_ok());
        assert!(validate_secret_name("MySecret123").is_ok());
        assert!(validate_secret_name("a").is_ok());
        assert!(validate_secret_name("API_KEY").is_ok());
    }

    #[test]
    fn test_invalid_secret_names() {
        assert!(validate_secret_name("").is_err());
        assert!(validate_secret_name("123secret").is_err()); // Starts with number
        assert!(validate_secret_name("-secret").is_err()); // Starts with hyphen
        assert!(validate_secret_name("secret-").is_err()); // Ends with hyphen
        assert!(validate_secret_name("my secret").is_err()); // Contains space
        assert!(validate_secret_name("my.secret").is_err()); // Contains period
    }

    #[test]
    fn test_max_length_passes() {
        // 255 characters should be valid (GCP max length)
        let name = format!("a{}", "b".repeat(254));
        assert_eq!(name.len(), 255);
        assert!(validate_secret_name(&name).is_ok());
    }

    #[test]
    fn test_over_max_length_fails() {
        // 256 characters should fail
        let name = format!("a{}", "b".repeat(255));
        assert_eq!(name.len(), 256);
        assert!(validate_secret_name(&name).is_err());
    }

    #[test]
    fn test_unicode_rejected() {
        // GCP requires ASCII-only secret names
        assert!(validate_secret_name("café").is_err());
        assert!(validate_secret_name("日本語").is_err());
        assert!(validate_secret_name("emoji🎉").is_err());
    }
}

/// Redact an email address for logs while keeping enough shape for debugging.
///
/// Examples:
/// - `sebastian@example.com` -> `s***@example.com`
/// - `a@example.com` -> `a***@example.com`
/// - malformed input -> `<redacted>`
pub fn redact_email(email: &str) -> String {
    let trimmed = email.trim();
    let Some((local, domain)) = trimmed.split_once('@') else {
        return "<redacted>".to_string();
    };
    if local.is_empty() || domain.is_empty() {
        return "<redacted>".to_string();
    }
    let first = local.chars().next().unwrap_or('*');
    format!("{}***@{}", first, domain)
}

#[cfg(test)]
mod tests {
    use super::redact_email;

    #[test]
    fn redacts_normal_email() {
        assert_eq!(redact_email("sebastian@example.com"), "s***@example.com");
    }

    #[test]
    fn redacts_short_local_part() {
        assert_eq!(redact_email("a@example.com"), "a***@example.com");
    }

    #[test]
    fn rejects_malformed_values() {
        assert_eq!(redact_email("not-an-email"), "<redacted>");
        assert_eq!(redact_email("@example.com"), "<redacted>");
        assert_eq!(redact_email("sebastian@"), "<redacted>");
    }
}

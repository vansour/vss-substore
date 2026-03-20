pub const APP_NAME: &str = "Submora";
pub const CURRENT_PHASE: u8 = 11;

const MAX_USERNAME_LENGTH: usize = 64;
const MAX_URL_LENGTH: usize = 2048;
const MIN_PASSWORD_LENGTH: usize = 8;

pub fn is_valid_username(username: &str) -> bool {
    if username.is_empty() || username.len() > MAX_USERNAME_LENGTH {
        return false;
    }

    username
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
}

pub fn is_valid_source_url(input: &str) -> bool {
    let trimmed = input.trim();
    if trimmed.is_empty() || trimmed.len() > MAX_URL_LENGTH {
        return false;
    }

    let Ok(parsed) = url::Url::parse(trimmed) else {
        return false;
    };

    matches!(parsed.scheme(), "http" | "https") && parsed.host_str().is_some()
}

pub fn is_valid_password_length(password: &str) -> bool {
    !password.is_empty() && password.len() <= 128
}

pub fn is_strong_password(password: &str) -> bool {
    if password.len() < MIN_PASSWORD_LENGTH {
        return false;
    }

    let has_letter = password.chars().any(|c| c.is_alphabetic());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_symbol = password.chars().any(|c| {
        matches!(
            c,
            '!' | '@'
                | '#'
                | '$'
                | '%'
                | '^'
                | '&'
                | '*'
                | '('
                | ')'
                | '_'
                | '+'
                | '-'
                | '='
                | '['
                | ']'
                | '{'
                | '}'
                | '|'
                | ';'
                | ':'
                | '\''
                | '"'
                | ','
                | '.'
                | '<'
                | '>'
                | '?'
                | '/'
                | '`'
                | '~'
        )
    });

    has_letter && has_digit && has_symbol
}

pub fn normalize_links_preserve_order(
    links: &[String],
    max_links: usize,
) -> Result<Vec<String>, String> {
    if links.len() > max_links {
        return Err(format!("maximum {max_links} allowed"));
    }

    let mut seen = std::collections::HashSet::new();
    let mut normalized = Vec::new();

    for raw in links {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }
        if !is_valid_source_url(trimmed) {
            return Err(format!("invalid url: {trimmed}"));
        }
        if seen.insert(trimmed.to_string()) {
            normalized.push(trimmed.to_string());
        }
    }

    Ok(normalized)
}


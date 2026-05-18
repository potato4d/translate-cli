use serde_json::Value;

use crate::error::{AppError, AppResult, EXIT_GENERAL};

pub fn normalize_allow_raw(raw: &str) -> AppResult<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(AppError::new(EXIT_GENERAL, "no translated_text in output"));
    }
    if let Some(text) = parse_json_translation(trimmed).or_else(|| parse_embedded_json(trimmed)) {
        return Ok(text);
    }
    Ok(trimmed.to_string())
}

fn parse_embedded_json(raw: &str) -> Option<String> {
    let starts = raw
        .match_indices('{')
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    let ends = raw
        .match_indices('}')
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    for start in starts.into_iter().rev() {
        for end in ends.iter().rev().copied() {
            if end <= start {
                continue;
            }
            if let Some(text) = parse_json_translation(&raw[start..=end]) {
                return Some(text);
            }
        }
    }
    None
}

fn parse_json_translation(raw: &str) -> Option<String> {
    let body = serde_json::from_str::<Value>(raw).ok()?;
    translated_text_from_value(&body)
}

fn translated_text_from_value(body: &Value) -> Option<String> {
    if let Some(text) = body.get("translated_text").and_then(Value::as_str) {
        return Some(text.to_string());
    }
    if let Some(text) = body
        .get("structured_output")
        .and_then(|value| value.get("translated_text"))
        .and_then(Value::as_str)
    {
        return Some(text.to_string());
    }
    if let Some(result) = body.get("result").and_then(Value::as_str) {
        if let Some(nested) = parse_json_translation(result) {
            return Some(nested);
        }
        let trimmed = result.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_structured_json() {
        assert_eq!(
            normalize_allow_raw(r#"{"structured_output":{"translated_text":"こんにちは"}}"#)
                .unwrap(),
            "こんにちは"
        );
    }

    #[test]
    fn normalizes_embedded_json() {
        assert_eq!(
            normalize_allow_raw(r#"noise {"translated_text":"hello"} tail"#).unwrap(),
            "hello"
        );
    }

    #[test]
    fn allows_raw_text() {
        assert_eq!(normalize_allow_raw(" hello ").unwrap(), "hello");
    }
}

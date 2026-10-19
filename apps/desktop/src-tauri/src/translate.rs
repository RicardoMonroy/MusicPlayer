//! Lyrics translation to Spanish through MyMemory's free API (no key).
//! Anonymous use allows ~5000 characters a day, so every result is cached on disk.
// shortcut: desktop-only for now; move to a shared crate when the TUI gets translations.

use std::path::PathBuf;

const API: &str = "https://api.mymemory.translated.net/get";
/// MyMemory rejects queries over 500 bytes.
const CHUNK_BYTES: usize = 450;

fn cache_file(uri: &str) -> Option<PathBuf> {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    uri.hash(&mut h);
    Some(
        dirs::cache_dir()?
            .join("musicplayer")
            .join("lyrics-es")
            .join(format!("{:016x}.txt", h.finish())),
    )
}

/// Whole lines packed into chunks under the API limit, so line breaks survive.
fn chunks(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for line in text.lines() {
        match out.last_mut() {
            Some(c) if c.len() + 1 + line.len() <= CHUNK_BYTES => {
                c.push('\n');
                c.push_str(line);
            }
            _ => out.push(line.to_string()),
        }
    }
    out
}

/// Spanish translation of `text` (lyrics of `uri`); the original when already Spanish.
pub async fn to_spanish(uri: &str, text: &str) -> Result<String, String> {
    let cached = cache_file(uri);
    if let Some(t) = cached
        .as_ref()
        .and_then(|f| std::fs::read_to_string(f).ok())
    {
        return Ok(t);
    }
    let http = reqwest::Client::new();
    let mut out = Vec::new();
    for chunk in chunks(text) {
        if chunk.trim().is_empty() {
            out.push(chunk);
            continue;
        }
        let res: serde_json::Value = http
            .get(API)
            .query(&[("q", chunk.as_str()), ("langpair", "autodetect|es")])
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;
        // responseStatus is a number on success and sometimes a string on errors.
        let status = res["responseStatus"]
            .as_i64()
            .or_else(|| res["responseStatus"].as_str()?.parse().ok());
        if status != Some(200) || res["quotaFinished"].as_bool() == Some(true) {
            let detail = res["responseDetails"]
                .as_str()
                .unwrap_or("error desconocido");
            return Err(
                if detail.contains("QUOTA") || res["quotaFinished"].as_bool() == Some(true) {
                    "Se alcanzó el límite diario de traducciones gratuitas; vuelve a intentarlo mañana.".into()
                } else {
                    format!("No se pudo traducir: {detail}")
                },
            );
        }
        if res["responseData"]["detectedLanguage"]
            .as_str()
            .is_some_and(|l| l.starts_with("es"))
        {
            return Ok(text.to_string());
        }
        out.push(
            res["responseData"]["translatedText"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
        );
    }
    let translated = out.join("\n");
    if let Some(f) = cached {
        let _ = std::fs::create_dir_all(f.parent().unwrap())
            .and_then(|_| std::fs::write(&f, &translated));
    }
    Ok(translated)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Hits MyMemory; run with `cargo test -p musicplayer-desktop -- --ignored`.
    #[tokio::test]
    #[ignore]
    async fn translates_and_keeps_lines() {
        let text = "I walk alone tonight\nThe wolves are calling me\n\nInto the cold";
        let es = to_spanish("test://translate", text).await.unwrap();
        assert_eq!(es.lines().count(), text.lines().count(), "{es}");
        assert_ne!(es, text);
        if let Some(f) = cache_file("test://translate") {
            let _ = std::fs::remove_file(f);
        }
    }

    #[test]
    fn chunks_keep_whole_lines_under_the_limit() {
        let line = "x".repeat(200);
        let text = format!("{line}\n{line}\n\n{line}");
        let c = chunks(&text);
        assert_eq!(c.len(), 2);
        assert!(c.iter().all(|c| c.len() <= CHUNK_BYTES));
        assert_eq!(c.join("\n"), text, "joining restores every line break");
    }
}

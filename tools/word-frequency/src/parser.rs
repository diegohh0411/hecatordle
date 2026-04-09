use std::path::Path;
use std::process::Command;

/// Extract all 5-letter lowercase ASCII words from a plain text string.
pub fn extract_words_from_text(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_ascii_alphabetic())
        .filter_map(|w| {
            let lower = w.to_ascii_lowercase();
            if lower.len() == 5 && lower.bytes().all(|b| b.is_ascii_lowercase()) {
                Some(lower)
            } else {
                None
            }
        })
        .collect()
}

/// Run Tesseract OCR on an image/PDF file and return extracted text.
/// Requires `tesseract` to be available on PATH.
pub fn ocr_file(path: &Path) -> Result<String, String> {
    let output = Command::new("tesseract")
        .arg(path)
        .arg("stdout")
        .arg("-l")
        .arg("eng")
        .output()
        .map_err(|e| format!("Failed to run tesseract: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Tesseract exited with error: {stderr}"));
    }

    String::from_utf8(output.stdout).map_err(|e| format!("Tesseract output is not UTF-8: {e}"))
}

/// Detect whether a file needs OCR based on its extension.
pub fn needs_ocr(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_ascii_lowercase())
            .as_deref(),
        Some("pdf") | Some("png") | Some("jpg") | Some("jpeg") | Some("tif") | Some("tiff")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extracts_five_letter_words() {
        let words = extract_words_from_text("The quick brown fox jumps over lazy dogs");
        assert!(words.contains(&"quick".to_string()));
        assert!(words.contains(&"brown".to_string()));
        assert!(words.contains(&"jumps".to_string()));
        assert!(!words.contains(&"the".to_string()));
        assert!(!words.contains(&"over".to_string()));
    }

    #[test]
    fn test_lowercases_words() {
        let words = extract_words_from_text("APPLE apple Apple");
        assert_eq!(words, vec!["apple", "apple", "apple"]);
    }

    #[test]
    fn test_ignores_non_ascii() {
        let words = extract_words_from_text("caf\u{e9} naïve hello");
        assert_eq!(words, vec!["hello"]);
    }

    #[test]
    fn test_needs_ocr() {
        assert!(needs_ocr(Path::new("book.pdf")));
        assert!(needs_ocr(Path::new("scan.png")));
        assert!(!needs_ocr(Path::new("book.txt")));
    }
}

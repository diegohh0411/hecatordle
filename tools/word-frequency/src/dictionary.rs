use rustc_hash::FxHashSet;
use std::fs;
use std::path::Path;
use indicatif::{ProgressBar, ProgressStyle};

const CACHE_PATH: &str = "scowl_words.txt";
const SCOWL_URL: &str =
    "https://raw.githubusercontent.com/dwyl/english-words/master/words_alpha.txt";

pub fn download_dictionary() -> FxHashSet<String> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner:.green} {msg}").unwrap());
    pb.enable_steady_tick(std::time::Duration::from_millis(120));

    let raw = if Path::new(CACHE_PATH).exists() {
        pb.set_message("Loading cached SCOWL wordlist...");
        fs::read_to_string(CACHE_PATH).unwrap_or_default()
    } else {
        pb.set_message("Downloading SCOWL wordlist...");
        match reqwest::blocking::get(SCOWL_URL).and_then(|r| r.text()) {
            Ok(content) => {
                let _ = fs::write(CACHE_PATH, &content);
                content
            }
            Err(e) => {
                pb.finish_with_message(format!("Failed to download SCOWL wordlist: {}", e));
                return FxHashSet::default();
            }
        }
    };

    let dict: FxHashSet<String> = raw
        .lines()
        .filter_map(|line| {
            let w = line.trim();
            if w.len() == 5 && w.bytes().all(|b| b.is_ascii_alphabetic()) {
                Some(w.to_owned())
            } else {
                None
            }
        })
        .collect();

    pb.finish_with_message(format!("Loaded {} valid 5-letter words.", dict.len()));
    dict
}

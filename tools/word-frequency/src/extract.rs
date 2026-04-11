use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use parquet::file::reader::{FileReader, SerializedFileReader};
use parquet::record::Field;

pub fn extract_words_from_parquet(path: &Path, mut on_word: impl FnMut(&str)) {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => { eprintln!("Warning: skipping Parquet {:?}: {}", path, e); return; }
    };
    let reader = match SerializedFileReader::new(file) {
        Ok(r) => r,
        Err(e) => { eprintln!("Warning: skipping Parquet {:?}: {}", path, e); return; }
    };
    let row_iter = match reader.get_row_iter(None) {
        Ok(iter) => iter,
        Err(e) => { eprintln!("Warning: skipping Parquet {:?}: {}", path, e); return; }
    };

    for row_result in row_iter {
        let row = match row_result {
            Ok(r) => r,
            Err(e) => { eprintln!("Warning: skipping row in {:?}: {}", path, e); continue; }
        };
        for (_, field) in row.get_column_iter() {
            match field {
                Field::Str(s) => {
                    for word in s.split_whitespace() {
                        on_word(word);
                    }
                }
                Field::Bytes(b) => {
                    if let Ok(s) = std::str::from_utf8(b.data()) {
                        for word in s.split_whitespace() {
                            on_word(word);
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

fn walk_json_strings(value: &serde_json::Value, on_word: &mut impl FnMut(&str)) {
    match value {
        serde_json::Value::String(s) => {
            for word in s.split_whitespace() {
                on_word(word);
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr { walk_json_strings(v, on_word); }
        }
        serde_json::Value::Object(map) => {
            for v in map.values() { walk_json_strings(v, on_word); }
        }
        _ => {}
    }
}

pub fn extract_words_from_json(path: &Path, mut on_word: impl FnMut(&str)) {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => { eprintln!("Warning: skipping JSON {:?}: {}", path, e); return; }
    };
    let reader = BufReader::new(file);
    let value: serde_json::Value = match serde_json::from_reader(reader) {
        Ok(v) => v,
        Err(e) => { eprintln!("Warning: skipping JSON {:?}: {}", path, e); return; }
    };
    walk_json_strings(&value, &mut on_word);
}

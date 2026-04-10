use std::fs;
use std::fs::File;
use std::path::Path;
use parquet::file::reader::{FileReader, SerializedFileReader};
use parquet::record::Field;
use lopdf::Document;

pub fn extract_text_from_pdf(path: &Path) -> String {
    let mut text = String::new();
    match Document::load(path) {
        Ok(doc) => {
            for (page_num, _) in doc.get_pages() {
                match doc.extract_text(&[page_num]) {
                    Ok(page_text) => { text.push_str(&page_text); text.push(' '); }
                    Err(e) => eprintln!("Warning: skipping page {} in {:?}: {}", page_num, path, e),
                }
            }
        }
        Err(e) => eprintln!("Warning: skipping PDF {:?}: {}", path, e),
    }
    text
}

pub fn extract_text_from_parquet(path: &Path) -> String {
    let mut text = String::new();

    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => { eprintln!("Warning: skipping Parquet {:?}: {}", path, e); return text; }
    };
    let reader = match SerializedFileReader::new(file) {
        Ok(r) => r,
        Err(e) => { eprintln!("Warning: skipping Parquet {:?}: {}", path, e); return text; }
    };
    let row_iter = match reader.get_row_iter(None) {
        Ok(iter) => iter,
        Err(e) => { eprintln!("Warning: skipping Parquet {:?}: {}", path, e); return text; }
    };

    for row_result in row_iter {
        let row = match row_result {
            Ok(r) => r,
            Err(e) => { eprintln!("Warning: skipping row in {:?}: {}", path, e); continue; }
        };
        for (_, field) in row.get_column_iter() {
            match field {
                Field::Str(s) => { text.push_str(s); text.push(' '); }
                Field::Bytes(b) => {
                    text.push_str(&String::from_utf8_lossy(b.data()));
                    text.push(' ');
                }
                _ => {}
            }
        }
    }
    text
}

fn collect_json_strings(value: &serde_json::Value, text: &mut String) {
    match value {
        serde_json::Value::String(s) => { text.push_str(s); text.push(' '); }
        serde_json::Value::Array(arr) => { for v in arr { collect_json_strings(v, text); } }
        serde_json::Value::Object(map) => { for v in map.values() { collect_json_strings(v, text); } }
        _ => {}
    }
}

pub fn extract_text_from_json(path: &Path) -> String {
    let mut text = String::new();
    match fs::read_to_string(path) {
        Ok(content) => match serde_json::from_str(&content) {
            Ok(value) => collect_json_strings(&value, &mut text),
            Err(e) => eprintln!("Warning: skipping JSON {:?}: {}", path, e),
        },
        Err(e) => eprintln!("Warning: skipping JSON {:?}: {}", path, e),
    }
    text
}

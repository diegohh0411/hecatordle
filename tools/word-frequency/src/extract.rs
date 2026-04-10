use std::fs;
use std::fs::File;
use std::path::Path;
use indicatif::ProgressBar;
use parquet::file::reader::{FileReader, SerializedFileReader};
use parquet::record::Field;

pub fn extract_text_from_parquet(path: &Path, pb: &ProgressBar) -> String {
    let mut text = String::new();

    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => { eprintln!("Warning: skipping Parquet {:?}: {}", path, e); return text; }
    };
    let reader = match SerializedFileReader::new(file) {
        Ok(r) => r,
        Err(e) => { eprintln!("Warning: skipping Parquet {:?}: {}", path, e); return text; }
    };

    let total_rows = reader.metadata().file_metadata().num_rows() as u64;
    pb.set_length(total_rows);

    let row_iter = match reader.get_row_iter(None) {
        Ok(iter) => iter,
        Err(e) => { eprintln!("Warning: skipping Parquet {:?}: {}", path, e); return text; }
    };

    let mut row_count: u64 = 0;
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
        row_count += 1;
        if row_count % 1_000 == 0 {
            pb.set_position(row_count);
        }
    }
    pb.set_position(row_count);
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

pub fn extract_text_from_json(path: &Path, pb: &ProgressBar) -> String {
    let mut text = String::new();
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => { eprintln!("Warning: skipping JSON {:?}: {}", path, e); return text; }
    };

    let total = content.bytes().filter(|&b| b == b'\n').count() as u64;
    pb.set_length(total.max(1));

    let stream = serde_json::Deserializer::from_str(&content).into_iter::<serde_json::Value>();
    let mut item_count: u64 = 0;
    for value_result in stream {
        match value_result {
            Ok(value) => collect_json_strings(&value, &mut text),
            Err(e) => eprintln!("Warning: skipping JSON item in {:?}: {}", path, e),
        }
        item_count += 1;
        pb.set_position(item_count);
    }
    text
}

use crate::frequency::WordData;
use dotenv::dotenv;

pub fn upsert_to_supabase(results: &[WordData]) {
    dotenv().ok();
    let url = std::env::var("SUPABASE_URL").ok();
    let key = std::env::var("SUPABASE_SERVICE_ROLE_KEY").ok();

    if let (Some(url), Some(key)) = (url, key) {
        let client = reqwest::blocking::Client::new();
        let endpoint = format!("{}/rest/v1/word_bank", url);

        // Destructive sync: deletes all rows then re-inserts.
        // If the process is interrupted mid-insert the table will be empty until the next run.
        println!("Clearing existing word_bank table...");
        let del = client.delete(format!("{}?word=not.is.null", endpoint))
            .header("apikey", &key)
            .header("Authorization", format!("Bearer {}", key))
            .header("Prefer", "return=minimal")
            .send();
        match del {
            Ok(resp) if !resp.status().is_success() => {
                println!("Error clearing table: {:?}", resp.text());
                return;
            }
            Err(e) => {
                println!("Failed to clear table: {}", e);
                return;
            }
            _ => println!("Table cleared."),
        }

        println!("Inserting {} words...", results.len());
        for chunk in results.chunks(1000) {
            let res = client.post(&endpoint)
                .header("apikey", &key)
                .header("Authorization", format!("Bearer {}", key))
                .header("Content-Type", "application/json")
                .header("Prefer", "return=minimal")
                .json(chunk)
                .send();

            match res {
                Ok(resp) if !resp.status().is_success() => {
                    println!("Error inserting chunk: {:?}", resp.text());
                }
                Err(e) => println!("Request failed: {}", e),
                _ => {}
            }
        }
        println!("Supabase sync complete.");
    } else {
        println!("Skipping Supabase sync: SUPABASE_URL or SUPABASE_SERVICE_ROLE_KEY not found in .env");
    }
}

use std::path::PathBuf;
use std::{env, fs};

use chrono::Local;
use kindle_to_notion::{clippings, notion};

fn main() {
    // TODO Move all the logic to `lib.rs` instead
    // For simplicity, the conf is read from a .env file at the moment
    dotenvy::dotenv().expect(".env file not found");

    // Reading the clippings
    let clippings_location: PathBuf = match env::var("CLIPPINGS_LOCATION") {
        Ok(clippings_file) => PathBuf::from(clippings_file),
        Err(_) => ["documents", "My Clippings.txt"].iter().collect(),
    };

    let clippings_text =
        fs::read_to_string(&clippings_location).expect("{file_path} file not found");

    // Creating our clips data
    let books_clips = clippings::parse_clips(clippings_text.as_str());

    println!("Found {} books with clips", books_clips.len());

    // Reading the environment variables for Notion
    let api_key = env::var("NOTION_API_KEY").expect("NOTION_API_KEY env variable not set");
    let parent_page_id = env::var("NOTION_PAGE_ID").expect("NOTION_PAGE_ID env variable not set");

    // Uploading to Notion
    notion::upload_clips(api_key.as_str(), parent_page_id.as_str(), books_clips)
        .expect("Failed to upload to Notion");

    // Archiving the clippings
    match env::var("DONT_ARCHIVE_CLIPPINGS") {
        Ok(no_archive) => {
            if no_archive == "true" {
                return;
            }
        }
        Err(_) => (),
    }

    fs::create_dir_all("clippings_archive").expect("Could not create clippings archive folder");
    let archive_location: PathBuf = [
        "clippings_archive",
        format!("{}.txt", Local::now()).as_str(),
    ]
    .iter()
    .collect();

    fs::rename(&clippings_location, &archive_location).expect("Could not archive clippings");
}

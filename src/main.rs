use std::fs::OpenOptions;
use std::path::PathBuf;
use std::{env, fs};

use kindle_to_notion::{clippings, notion};
use std::io::prelude::*;

fn main() {
    // For simplicity, the conf is read from a .env file at the moment
    dotenvy::dotenv().expect(".env file not found");

    // Getting the clippings location
    let clippings_location: PathBuf = env::var("CLIPPINGS_LOCATION").map_or_else(
        // Use documents/My Clippings.txt as default
        |_| ["documents", "My Clippings.txt"].iter().collect(),
        // Otherwise, use the env variable (which is OS-specific)
        PathBuf::from,
    );

    // Reading the clippings
    let clippings_text = fs::read_to_string(&clippings_location).expect("Clippings file not found");

    // Creating our clips data
    let books_clips = clippings::parse_clips(clippings_text.as_str());

    println!("Found {} books with new clips", books_clips.len());

    // Reading the environment variables for Notion
    let api_key = env::var("NOTION_API_KEY").expect("NOTION_API_KEY env variable not set");
    let parent_page_id = env::var("NOTION_PAGE_ID").expect("NOTION_PAGE_ID env variable not set");

    // Uploading to Notion
    notion::upload_clips(api_key.as_str(), parent_page_id.as_str(), books_clips)
        .expect("Failed to upload to Notion");

    // Marking the end of the clippings
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(&clippings_location)
        .expect("Could not open clippings file as appendable");

    writeln!(file, "==========").expect("Could not write to clippings file");
}

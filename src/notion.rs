use std::str::FromStr;

use crate::clippings::BookClips;
use anyhow::Result;
use reqwest::header::{HeaderMap, HeaderName};
use reqwest::{self, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

const NOTION_API_URL: &str = "https://api.notion.com/v1/pages";

const NOTION_API_VERSION_HEADER: &str = "Notion-Version";
const NOTION_API_VERSION: &str = "2022-06-28";

#[derive(Debug, Serialize, Deserialize)]
struct NotionBookPage {
    parent: Value,
    icon: Value,
    properties: Value,
    children: Vec<Value>,
}

// Creating our clip function that uses the parent
impl BookClips {
    fn to_notion_body(&self, parent_page_id: String) -> NotionBookPage {
        // TODO Use strongly typed JSON everywhere to make it easier, it's a bit disgusting here
        let mut children = Vec::new();

        children.push(json!({
            "object": "block",
            "type": "callout",
            "callout": {
                "rich_text": [
                    {
                        "type": "text",
                        "text": {
                            "content": self.author,
                        },
                    }
                ],
                "icon": {
                    "emoji": "✍️"
                },
                "color": "default"
            }
        }));

        children.push(json!({
            "object": "block",
            "type": "divider",
            "divider": {}
        }));

        for clip in &self.clips {
            let mut split_content = Vec::new();
            let mut current_content = String::new();

            // RE-CHECK WHY THIS IS NEEDED, 2000 characters is HUGE wtf
            for phrase in clip.content.split_inclusive(". ") {
                // We split every 1800 characters to leave space for the date
                if current_content.len() + phrase.len() > 1800 {
                    split_content.push(current_content);
                    current_content = String::from(phrase);
                // Else we just grow the content and add the dot back
                } else {
                    current_content.push_str(phrase);
                }
            }

            // Adding the remainder (usually the whole content)
            split_content.push(current_content);

            // We iterate on content blocks
            for (idx, content) in split_content.iter().enumerate() {
                let quote_content = if idx < split_content.len() - 1 {
                    // First part of quote: no line jump or date
                    json!([{
                        "type": "text",
                        "text": {
                            "content": format!("{}", content)
                            }
                    }])
                } else {
                    // Second part of quote: line jump and date
                    json!([{
                        "type": "text",
                        "text": {
                            "content": format!("{}\n", content)
                            }
                    },
                    {
                        "type": "mention",
                        "mention":{
                            "type": "date",
                            "date": {
                                "start": clip.date,
                            }
                        }
                    }
                    ])
                };

                children.push(json!({
                    "object": "block",
                    "type": "quote",
                    "quote": {
                        "rich_text": quote_content
                        }
                    }
                ))
            }
        }

        NotionBookPage {
            parent: json!({ "page_id": parent_page_id }),
            icon: json!({
                "emoji": "📖"
            }),
            properties: json!({
                "title": [
                        {
                            "text": {
                                "content": self.book_name
                            }
                        }
                    ]
            }),
            children,
        }
    }
}

pub fn upload_to_notion(
    api_key: String,
    parent_page_id: String,
    books_clips: Vec<BookClips>,
) -> Result<()> {
    let client = reqwest::blocking::Client::new();

    // TODO Cleanup if that works
    let mut headers = HeaderMap::new();
    let header_name = HeaderName::from_str(NOTION_API_VERSION_HEADER)?;
    headers.insert(header_name, NOTION_API_VERSION.parse()?);

    for book in books_clips {
        println!("Uploading clips from {:?}", book.book_name);

        let res = client
            .post(NOTION_API_URL)
            .bearer_auth(api_key.clone())
            .headers(headers.clone())
            .json(&book.to_notion_body(parent_page_id.clone()))
            .send()?;

        match res.status() {
            StatusCode::OK => {
                continue;
            }
            StatusCode::BAD_REQUEST => {
                println!("Bad request: {:?}", res.text()?);
            }
            _ => {
                println!("Unexpected status code: {:?}", res.status());
            }
        }
    }

    Ok(())
}

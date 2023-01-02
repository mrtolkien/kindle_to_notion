use crate::clippings::BookClips;
use anyhow::Result;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::{self, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

const NOTION_API_URL: &str = "https://api.notion.com/v1/pages";

#[derive(Debug, Serialize, Deserialize)]
struct NotionBookPage {
    parent: Value,
    icon: Value,
    properties: Value,
    children: Vec<Value>,
}

// Creating our clip function that uses the parent
impl BookClips {
    fn to_notion_body(&self, parent_page_id: &str) -> NotionBookPage {
        // TODO Use strongly typed JSON everywhere to make it easier, it's a bit disgusting here
        let mut children = Vec::new();

        // We split on : if it's in the name, as it's usually ridiculously long books names then
        let page_name = if self.book_name.split(':').count() == 1 {
            // If there's no : in the name, it's simply the book's name
            &self.book_name
        } else {
            // We add the full name as callout
            children.push(json!({
                "object": "block",
                "type": "callout",
                "callout": {
                    "rich_text": [
                        {
                            "type": "text",
                            "text": {
                                "content": self.book_name,
                            },
                        }
                    ],
                    "icon": {
                        "emoji": "📕"
                    },
                    "color": "default"
                }
            }));

            // We return the first part of the string
            self.book_name
                .split(':')
                .next()
                .unwrap_or_else(|| unreachable!())
        };

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
                            "content": format!("{content}")
                            }
                    }])
                } else {
                    // Second part of quote: line jump and date
                    json!([{
                        "type": "text",
                        "text": {
                            "content": format!("{content}\n")
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
                ));
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
                                "content": page_name
                            }
                        }
                    ]
            }),
            children,
        }
    }
}

/// Uploads the book clips to Notion
///
/// # Arguments
///
/// * `api_key` - The Notion API key
/// * `parent_page_id` - The ID of the parent page where the clips pages will be created
/// * `books_clips` - The list of book clips to upload
///
/// # Errors
/// Raise on HTTP errors from the API call to Notion
pub fn upload_clips(
    api_key: &str,
    parent_page_id: &str,
    books_clips: Vec<BookClips>,
) -> Result<()> {
    let client = reqwest::blocking::Client::new();

    // Defining custom headers
    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("notion-version"),
        HeaderValue::from_static("2022-06-28"),
    );

    for book in books_clips {
        println!("Uploading clips from {:?}", book.book_name);

        let res = client
            .post(NOTION_API_URL)
            .bearer_auth(api_key)
            .headers(headers.clone())
            .json(&book.to_notion_body(parent_page_id))
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

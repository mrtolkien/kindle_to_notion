use crate::clippings::BookClips;
use anyhow::Result;
use chrono::{DateTime, Local};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::{self, StatusCode};
use serde::{Deserialize, Serialize};

const NOTION_API_URL: &str = "https://api.notion.com/v1/pages";

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

// Creating our clip function that uses the parent
impl BookClips {
    fn to_notion_body(&self, parent_page_id: &str) -> NotionPageQuery {
        let mut children = Vec::new();

        // We split on : if it's in the name, as it's usually ridiculously long books names then
        let page_name = if self.book_name.split(':').count() == 1 {
            // If there's no : in the name, it's simply the book's name
            &self.book_name
        } else {
            children.push(Child::new_callout(self.book_name.to_string(), "📕"));

            // We return the first part of the string
            self.book_name
                .split(':')
                .next()
                .unwrap_or_else(|| unreachable!())
        };

        // Adding the author
        children.push(Child::new_callout(self.author.to_string(), "✍️"));
        children.push(Child::new_divider());

        // Adding clips
        for clip in &self.clips {
            let mut split_content = Vec::new();
            let mut current_content = String::new();

            // We split the content on dots to check if they're very long
            // If they are, we split them in multiple blocks because there's a 2000 characters limit
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

            // Adding the remainder (aka the whole content if no split)
            split_content.push(current_content);

            // We iterate on content blocks
            for (idx, content) in split_content.iter().enumerate() {
                if idx < split_content.len() - 1 {
                    // First part of quote: no line jump or date
                    children.push(Child::new_quote(content.to_string(), None));
                } else {
                    // Second part of quote: line jump and date
                    children.push(Child::new_quote(content.to_string(), Some(clip.date)));
                };
            }
        }

        NotionPageQuery {
            parent: Parent {
                page_id: parent_page_id.to_string(),
            },
            icon: Icon {
                emoji: "📖".to_string(),
            },
            properties: Properties {
                title: vec![Title {
                    text: Text {
                        content: page_name.to_string(),
                    },
                }],
            },
            children,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct NotionPageQuery {
    parent: Parent,
    icon: Icon,
    properties: Properties,
    children: Vec<Child>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct Parent {
    page_id: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct Properties {
    title: Vec<Title>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct Title {
    text: Text,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct Child {
    pub object: ObjectType,
    #[serde(rename = "type")]
    pub type_field: BlockType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callout: Option<Callout>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub divider: Option<Divider>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote: Option<Quote>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ObjectType {
    #[default]
    Block,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum BlockType {
    Callout,
    Divider,
    #[default]
    Quote,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Divider {}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Callout {
    pub color: Color,
    pub icon: Icon,
    pub rich_text: Vec<RichText>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Quote {
    pub rich_text: Vec<RichText>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Color {
    #[default]
    Default,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Icon {
    pub emoji: String,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RichText {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<Text>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mention: Option<Mention>,
    #[serde(rename = "type")]
    pub type_field: TextType,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Text {
    pub content: String,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mention {
    pub date: Option<Date>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Date {
    pub start: DateTime<Local>,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TextType {
    #[default]
    Text,
    Mention,
}

impl Child {
    pub fn new_callout(content: String, emoji: &str) -> Self {
        let mut child = Self {
            type_field: BlockType::Callout,
            ..Default::default()
        };

        let mut callout = Callout {
            icon: Icon {
                emoji: emoji.to_string(),
            },
            ..Default::default()
        };
        callout.rich_text.push(RichText {
            text: Some(Text { content }),
            ..Default::default()
        });

        child.callout = Some(callout);

        child
    }

    pub fn new_divider() -> Self {
        let mut child = Self {
            type_field: BlockType::Divider,
            ..Default::default()
        };
        child.divider = Some(Divider::default());

        child
    }

    pub fn new_quote(content: String, date: Option<DateTime<Local>>) -> Self {
        let mut child = Self::default();

        let mut quote = Quote {
            rich_text: vec![RichText {
                text: Some(Text { content }),
                ..Default::default()
            }],
        };

        if let Some(date) = date {
            quote.rich_text.push(RichText {
                text: Some(Text {
                    content: "\n".to_string(),
                }),
                ..Default::default()
            });
            quote.rich_text.push(RichText {
                type_field: TextType::Mention,
                mention: Some(Mention {
                    date: Some(Date { start: date }),
                }),
                ..Default::default()
            });
        };

        child.quote = Some(quote);

        child
    }
}

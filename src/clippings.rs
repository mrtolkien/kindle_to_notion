use chrono::NaiveDateTime;
use nom::{
    bytes::complete::{tag, take, take_until},
    character::complete::{digit1, line_ending, not_line_ending},
    combinator::map_parser,
    multi::{many_till, separated_list0},
    sequence::{delimited, terminated, tuple},
    IResult,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct BookClips {
    pub book_name: String,
    pub author: String,
    pub clips: Vec<Clip>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Clip {
    pub book: String,
    pub author: String,
    pub content: String,
    pub date: NaiveDateTime,
    // Start/End locations
    pub location: (usize, usize),
}

/// Parses a Kindle clippings file into a vector of `BookClips`
/// # Variables
/// * `input` - The input string to parse
/// # Returns
/// * `Vec<BookClips>` - The parsed clippings
/// # Example
/// ```
/// use kindle_to_notion::clippings;
/// let clippings_text = "The Lord of the Rings (J. R. R. Tolkien)
/// - Your Highlight on Location 1234-1235 | Added on Monday, 1 January 2021 00:00:00
///
/// This is a clip
/// ==========
/// The Lord of the Rings (J. R. R. Tolkien)
/// - Your Highlight on Location 5678-5678 | Added on Tuesday, 2 January 2021 00:00:00
///
/// This is another clip";
///
/// let books_clips = clippings::parse_clips(clippings_text);
/// ```
pub fn parse_clips(input: &str) -> Vec<BookClips> {
    let (_, clips) =
        separated_list0(tuple((tag("=========="), line_ending)), nom_single_clip)(input)
            .expect("Could not parse clippings");

    clips
        .group_by(|a, b| a.book == b.book && a.author == b.author)
        .map(|clips| BookClips {
            book_name: clips[0].book.clone(),
            author: clips[0].author.clone(),
            clips: Vec::from(clips),
        })
        .collect()
}

/// Uses nom to parse a single clip, delimited by `==========`
/// # Variables
/// * `input` - The input string to parse
///   * Example:
/// ```text
/// The Lord of the Rings (J. R. R. Tolkien)
///  - Your Highlight on Location 1234-1235 | Added on Monday, 1 January 2021 00:00:00
///
/// This is a clip
/// ```
/// # Returns
/// * `IResult<&str, Clip>` - Input remainder + The parsed clip struct
fn nom_single_clip(input: &str) -> IResult<&str, Clip> {
    let (input, ((book, author), (location_start, _, location_end), _, date, content)) =
        tuple((
            nom_first_row,
            delimited(
                tag("- Your Highlight at location "),
                tuple((digit1, take(1usize), digit1)),
                tag(" |"),
            ),
            take_until(", "),
            map_parser(
                delimited(
                    tag(", "),
                    not_line_ending,
                    tuple((line_ending, line_ending)),
                ),
                parse_date,
            ),
            terminated(not_line_ending, line_ending),
        ))(input)?;

    Ok((
        input,
        Clip {
            book,
            author: author.replace(')', ""),
            content: content.to_string(),
            date,
            location: (
                location_start.parse().expect("Not a valid integer"),
                location_end.parse().expect("Not a valid integer"),
            ),
        },
    ))
}

/// Uses nom to parse the first row of a clip, which contains the book name and the author
/// # Variables
/// * `input` - The input string to parse
///    * Example: `The Lord of the Rings (J. R. R. Tolkien)`
/// # Returns
/// * `IResult<&str, (String, &str)>` - Input remainder + The parsed book name and author
pub fn nom_first_row(input: &str) -> IResult<&str, (String, &str)> {
    let (input, (book, author)) = many_till(
        take(1_usize),
        delimited(tag(" ("), take_until(")"), tuple((tag(")"), line_ending))),
    )(input)?;

    Ok((
        input,
        (book.iter().fold(String::new(), |acc, x| acc + x), author),
    ))
}

/// Parses a date from the format `1 January 2021 00:00:00`
/// # Variables
/// * `input` - The input string to parse
///   * Example: `1 January 2021 00:00:00`
/// # Returns
/// * `IResult<&str, String>` - Input remainder + The parsed date in the format `YYYY-MM-DD`
/// # Note
/// We use the IResult type here to integrate with nom
fn parse_date(input: &str) -> IResult<&str, NaiveDateTime> {
    Ok((
        "",
        NaiveDateTime::parse_from_str(input, "%e %B %Y %H:%M:%S").expect("Cannot parse input"),
    ))
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::*;
    use std::fs;

    fn get_test_clippings() -> String {
        fs::read_to_string("tests/data/clippings.txt").expect("Test file not found")
    }

    #[test]
    fn test_parse_date() {
        let test_date = "1 December 2020 16:58:58";

        let (_, parsed_date) = parse_date(test_date).unwrap();

        assert_eq!(
            parsed_date,
            NaiveDate::from_ymd_opt(2020, 12, 1)
                .unwrap()
                .and_hms_micro_opt(16, 58, 58, 0)
                .unwrap()
        );
    }

    #[test]
    fn test_parse_first_row() {
        let first_row = "Building... (NEW) (2022) (Tiago Forte)
";
        let (_, (author, book)) = nom_first_row(first_row).unwrap();

        assert_eq!(author, "Building... (NEW) (2022)");
        assert_eq!(book, "Tiago Forte");
    }

    #[test]
    fn test_parse_single_clip_simple() {
        let test_clip = "How to Win Friends and Influence People (Dale Carnegie)
- Your Highlight at location 1502-1507 | Added on Tuesday, 1 December 2020 16:58:58

The old neighbour called at the White House, and Lincoln talked to him for hours about the advisability of issuing a proclamation freeing the slaves. Lincoln went over all the arguments for and against such a move, and then read letters and newspaper articles, some denouncing him for not freeing the slaves and others denouncing him for fear he was going to free them. After talking for hours, Lincoln shook hands with his old neighbour, said good night, and sent him back to Illinois without even asking for his opinion. Lincoln had done all the talking himself. That seemed to clarify his mind. ‘He seemed to feel easier after that talk,’ the old friend said. Lincoln hadn’t wanted advice. He had wanted merely a friendly, sympathetic listener to whom he could unburden himself.
";
        let (_, parsed_clip) = nom_single_clip(test_clip).unwrap();
        insta::assert_yaml_snapshot!(parsed_clip);
    }

    #[test]
    fn test_parse_single_clip_parenthesis_in_title() {
        let test_clip = "Building a Second Brain: A Proven Method to Organize Your Digital Life and Unlock Your Creative Potential (2022) (Tiago Forte)
- Your Highlight at location 1096-1097 | Added on Sunday, 18 December 2022 10:20:38

It’s important to keep capturing relatively effortless because it is only the first step.
";
        let (_, parsed_clip) = nom_single_clip(test_clip).unwrap();
        insta::assert_yaml_snapshot!(parsed_clip);
    }

    #[test]
    fn test_all_clippings_parsing() {
        let input = get_test_clippings();
        let parsed_clippings = parse_clips(input.as_str());
        insta::assert_yaml_snapshot!(parsed_clippings);
    }
}

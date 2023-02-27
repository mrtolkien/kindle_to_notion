use chrono::{DateTime, Local, TimeZone};
use nom::{
    bytes::complete::{tag, take, take_until},
    character::complete::{digit1, line_ending, not_line_ending},
    combinator::map,
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
    pub date: DateTime<Local>,
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
    // We use ==========\n========== to mark previously finished parsing jobs
    // So we split on this marker and take everything after it
    let input = input
        .split("#==========")
        .last()
        .unwrap_or_else(|| unreachable!("A string is always splittable"));

    // We parse all the clips with nom
    let (_, clips) =
        separated_list0(tuple((tag("=========="), line_ending)), nom_single_clip)(input)
            .expect("Could not parse clippings");

    // We group clips by book and author
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
/// # Errors
/// * `IResult::Error` - If the input cannot be parsed
fn nom_single_clip(input: &str) -> IResult<&str, Clip> {
    let (input, ((book, author), (location_start, location_end), _, date, content)) = tuple((
        // Book name and author
        nom_first_row,
        // Location
        nom_location_2023_02,
        // Removing the " | Added on " part
        take_until(", "),
        // Date
        map(
            delimited(
                tag(", "),
                not_line_ending,
                tuple((line_ending, line_ending)),
            ),
            parse_date,
        ),
        // Content
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
/// # Errors
/// * `IResult::Error` - If the input cannot be parsed
pub fn nom_first_row(input: &str) -> IResult<&str, (String, &str)> {
    let (input, (book, author)) = many_till(
        take(1_usize),
        delimited(tag(" ("), take_until(")"), tuple((tag(")"), line_ending))),
    )(input)?;

    Ok((
        input,
        (
            book.iter()
                // Removing BOM character and line jumps if present
                .filter(|x| **x != '\u{feff}'.to_string() && **x != '\n'.to_string())
                .fold(String::new(), |acc, x| acc + x),
            author,
        ),
    ))
}

/// Uses nom to parse the location of a clip
/// -> OBSOLETE and not used anymore atm
/// # Variables
/// * `input` - The input string to parse
///  * Example: - Your Highlight at location 1502-1507 |
/// # Returns
/// * `IResult<&str, (&str, &str)>` - Input remainder + The parsed start and end location
/// # Errors
/// * `IResult::Error` - If the input cannot be parsed
pub fn nom_location_old(input: &str) -> IResult<&str, (&str, &str)> {
    let (input, (location_start, _, location_end)) = delimited(
        tag("- Your Highlight at location "),
        tuple((digit1, take(1usize), digit1)),
        tag(" |"),
    )(input)?;

    Ok((input, (location_start, location_end)))
}

/// Uses nom to parse the location of a clip, updated format
/// # Variables
/// * `input` - The input string to parse
///  * Example: - Your Highlight at location 1502-1507 |
/// # Returns
/// * `IResult<&str, (&str, &str)>` - Input remainder + The parsed start and end location
/// # Errors
/// * `IResult::Error` - If the input cannot be parsed
pub fn nom_location_2023_02(input: &str) -> IResult<&str, (&str, &str)> {
    // Removing the page
    let (input, _) = take_until("location")(input)?;

    let (input, (location_start, _, location_end)) = delimited(
        tag("location "),
        tuple((digit1, take(1usize), digit1)),
        tag(" |"),
    )(input)?;

    Ok((input, (location_start, location_end)))
}

/// Parses a date from the format `1 January 2021 00:00:00`
/// # Variables
/// * `input` - The input string to parse
///   * Example: `1 January 2021 00:00:00`
/// # Returns
/// * `IResult<&str, String>` - Input remainder + The parsed date in the format `YYYY-MM-DD`
/// # Note
fn parse_date(input: &str) -> DateTime<Local> {
    Local
        .datetime_from_str(input, "%e %B %Y %H:%M:%S")
        .expect("Cannot parse input")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn get_test_clippings() -> String {
        fs::read_to_string("tests/data/clippings.txt").expect("Test file not found")
    }

    #[test]
    fn test_parse_date() {
        let test_date = "1 December 2020 16:58:58";

        let parsed_date = parse_date(test_date);

        assert_eq!(
            parsed_date,
            Local.with_ymd_and_hms(2020, 12, 1, 16, 58, 58).unwrap()
        );
    }

    #[test]
    fn test_parse_first_row() {
        let first_row = "Building... (NEW) (2022) (Tiago Forte)
";
        let (_, (author, book)) = nom_first_row(first_row).expect("Could not nom first row");

        assert_eq!(author, "Building... (NEW) (2022)");
        assert_eq!(book, "Tiago Forte");
    }

    #[test]
    fn test_parse_single_clip_simple() {
        let test_clip = "How to Win Friends and Influence People (Dale Carnegie)
- Your Highlight at location 1502-1507 | Added on Tuesday, 1 December 2020 16:58:58

The old neighbour called at the White House, and Lincoln talked to him for hours about the advisability of issuing a proclamation freeing the slaves. Lincoln went over all the arguments for and against such a move, and then read letters and newspaper articles, some denouncing him for not freeing the slaves and others denouncing him for fear he was going to free them. After talking for hours, Lincoln shook hands with his old neighbour, said good night, and sent him back to Illinois without even asking for his opinion. Lincoln had done all the talking himself. That seemed to clarify his mind. ‘He seemed to feel easier after that talk,’ the old friend said. Lincoln hadn’t wanted advice. He had wanted merely a friendly, sympathetic listener to whom he could unburden himself.
";
        let (_, parsed_clip) = nom_single_clip(test_clip).expect("Could not nom clip");
        insta::assert_yaml_snapshot!(parsed_clip);
    }

    #[test]
    fn test_parse_single_clip_parenthesis_in_title() {
        let test_clip = "Building a Second Brain: A Proven Method to Organize Your Digital Life and Unlock Your Creative Potential (2022) (Tiago Forte)
- Your Highlight at location 1096-1097 | Added on Sunday, 18 December 2022 10:20:38

It’s important to keep capturing relatively effortless because it is only the first step.
";
        let (_, parsed_clip) =
            nom_single_clip(test_clip).expect("Could not nom clip with parenthesis in title");
        insta::assert_yaml_snapshot!(parsed_clip);
    }

    #[test]
    fn test_parse_single_clip_2023_02_format() {
        let test_clip = "Shoe Dog (Phil Knight)
- Your Highlight on page 58 | location 877-879 | Added on Monday, 13 February 2023 00:29:40

People reflexively assume that competition is always a good thing, that it always brings out the best in people, but that’s only true of people who can forget the competition. The art of competing, I’d learned from track, was the art of forgetting, and I now reminded myself of that fact. You must forget your limits.
";
        let (_, parsed_clip) =
            nom_single_clip(test_clip).expect("Could not nom clip with new format");
        insta::assert_yaml_snapshot!(parsed_clip);
    }

    #[test]
    fn test_parse_location_2023_02() {
        let test_location = "- Your Highlight on page 58 | location 877-879 |";

        let (_, parsed_location) =
            nom_location_2023_02(test_location).expect("Could not nom location with new format");
        insta::assert_yaml_snapshot!(parsed_location);
    }

    #[test]
    fn test_all_clippings_parsing() {
        let input = get_test_clippings();
        let parsed_clippings = parse_clips(input.as_str());
        insta::assert_yaml_snapshot!(parsed_clippings);
    }
}

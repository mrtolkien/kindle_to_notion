use nom::{
    bytes::complete::{tag, take, take_until},
    character::complete::{alpha1, digit1, line_ending, not_line_ending},
    multi::{many_till, separated_list0},
    sequence::{delimited, terminated, tuple},
    IResult,
};
use phf::phf_map;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Clip {
    book: String,
    author: String,
    content: String,
    date: String,
}

pub fn parse_clips(input: &str) -> Vec<Clip> {
    let (_, result) = separated_list0(tag("==========\n"), nom_single_clip)(input)
        .expect("Could not parse clippings");

    result
}

fn nom_single_clip(input: &str) -> IResult<&str, Clip> {
    let (input, ((book, author), _, raw_date, content)) = tuple((
        nom_first_row,
        take_until(", "),
        delimited(
            tag(", "),
            not_line_ending,
            tuple((line_ending, line_ending)),
        ),
        terminated(not_line_ending, line_ending),
    ))(input)?;

    // Parsing the date
    let (_, date) = parse_date(raw_date)?;

    Ok((
        input,
        Clip {
            book,
            author: author.replace(')', ""),
            content: content.to_string(),
            date,
        },
    ))
}

fn nom_first_row(input: &str) -> IResult<&str, (String, &str)> {
    let (input, (book, author)) = many_till(
        take(1_usize),
        delimited(tag(" ("), take_until(")"), tag(")\n")),
    )(input)?;

    Ok((
        input,
        (book.iter().fold(String::new(), |acc, x| acc + x), author),
    ))
}

static MONTHS: phf::Map<&'static str, u8> = phf_map! {
    "January" => 1,
    "February" => 2,
    "March" => 3,
    "April" => 4,
    "May" => 5,
    "June" => 6,
    "July" => 7,
    "August" => 8,
    "September" => 9,
    "October" => 10,
    "November" => 11,
    "December" => 12,
};

fn parse_date(input: &str) -> IResult<&str, String> {
    let (input, day) = digit1(input)?;

    let day = if day.len() == 1 {
        // We pad with a 0
        format!("0{day}")
    } else {
        day.to_string()
    };

    let (input, month) = delimited(tag(" "), alpha1, tag(" "))(input)?;
    let (_, year) = digit1(input)?;

    Ok((
        "",
        format!(
            "{}-{}-{}",
            year,
            MONTHS.get(month).expect("Month {month} not found"),
            day
        ),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    // TODO Check insta VS code extension: https://marketplace.visualstudio.com/items?itemName=mitsuhiko.insta
    fn get_test_clippings() -> String {
        fs::read_to_string("tests/data/clippings.txt").expect("Test file not found")
    }

    #[test]
    fn test_parse_date() {
        let test_date = "1 December 2020 16:58:58";
        let (_, parsed_date) = parse_date(test_date).unwrap();

        assert_eq!(parsed_date, "2020-12-01");
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

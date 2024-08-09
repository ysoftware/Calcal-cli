#![allow(dead_code)] // todo: remove
#![allow(unused_imports)] // todo: remove
#![allow(unused_variables)] // todo: remove

use std::process::exit;
use std::fmt;

fn main() {
    test_advance_if_possible_after_unicode();
    test_next_matches_ascii();
    test_first_index();
    println!("---- start program ------");

    println!("Hello world!\n");

    let response = make_http_request().unwrap_or_else(|error| {
        eprintln!("An error occured while making http request: {error}");
        exit(1)
    });

    parse_entities(response).unwrap_or_else(|error| {
        eprintln!("An error occured while parsing response: {error}");
        exit(1)
    });
}

fn parse_entities(string: String) -> Result<Vec<EntryEntity>, Error> {
    let mut parser = Parser {
        i: 0,
        end_index: string.len(),
        text: string,
        entries: vec![],
    };

    while parser.i < parser.text.len() {
        eat_whitespaces_and_newlines(&mut parser);

        let index_after_date = first_index(&parser, ' ');
        if !next_matches_ascii(&parser, "Date: ") && index_after_date != NOT_FOUND {
            print_error_position(&parser);
            return Err(Error::ExpectedEntry)
        }

        println!("Found entry!");

        // guard textRemainder[i..<endIndex].starts(with: "Date: "),
        //       let indexAfterDate = textRemainder.firstIndex(of: " ")
        // else {
        //     printErrorPosition()
        //     throw Error.expectedEntry
        // }
        // advance_if_possible_after_unicode(after: indexAfterDate)

        // eatWhitespaces()
        // guard let dateNewLineIndex = textRemainder.firstIndex(of: "\n")
        // else {
        //     printErrorPosition()
        //     throw Error.expectedEOF
        // }
        // let dateString = String(textRemainder[i..<dateNewLineIndex]).trimmingCharacters(in: .whitespaces)
        // advance_if_possible_after_unicode(after: dateNewLineIndex)

        // var sections: [EntryEntity.Section] = []




        // for debugging
        parser.i += 1;
    }


    todo!()
}

fn first_index(parser: &Parser, search: char) -> i32 {
    first_index_s(parser.text.as_bytes(), parser.i, parser.end_index, search)
}
fn first_index_s(bytes: &[u8], offset: usize, end_index: usize, search: char) -> i32 {
    let mut i = offset;
    while end_index > i && bytes[i] as char != search {
        advance_if_possible_after_unicode_s(bytes, &mut i, end_index, 0);
    }
    return NOT_FOUND
}

fn next_matches_ascii(parser: &Parser, search: &str) -> bool {
    next_matches_ascii_s(parser.text.as_bytes(), parser.i, parser.end_index, search)
}
fn next_matches_ascii_s(bytes: &[u8], i: usize, end_index: usize, search: &str) -> bool {
    let remaining_length = end_index - i;

    let search_length: usize = search.len();
    if remaining_length < search_length {
        return false
    }

    let byte_slice = &bytes[i..i + search_length];
    return byte_slice == search.as_bytes()
}

fn print_error_position(parser: &Parser) {
    let previous_symbols = &parser.text[parser.i..std::cmp::min(parser.i + 100, parser.end_index)];
    println!("Parser: Error occured right before this text:\n{}", previous_symbols);
}

fn eat_whitespaces_but_no_newlines(parser: &mut Parser) {
    while parser.end_index > parser.i && is_whitespace_but_not_newline(parser) {
        let i_value = parser.i.clone(); // bad code, how to write it inline?
        advance_if_possible_after_unicode(parser, i_value);
    }
}

fn eat_whitespaces_and_newlines(parser: &mut Parser) {
    while parser.end_index > parser.i && is_whitespace(parser) {
        let i_value = parser.i.clone();
        advance_if_possible_after_unicode(parser, i_value);
    }
}

fn is_whitespace_but_not_newline(parser: &Parser) -> bool {
    let c = parser.text.as_bytes()[parser.i] as char;
    return c == ' ' || c != '\n'
}

fn is_whitespace(parser: &Parser) -> bool {
    let c = parser.text.as_bytes()[parser.i] as char;
    return c == ' ' || c == '\n'
}

fn advance_if_possible_after_unicode(parser: &mut Parser, after: usize) {
    advance_if_possible_after_unicode_s(parser.text.as_bytes(), &mut parser.i, parser.end_index, after);
}
fn advance_if_possible_after_unicode_s(text: &[u8], i: &mut usize, end_index: usize, after: usize) {
    if end_index < after {
        *i = end_index;
        return
    }

    let mut characters_count = 0;

    while *i < end_index {
        if characters_count > after {
            break
        }

        let first_byte = text[*i];
        let advance_bytes: usize;
        
        // figure out how many bytes long is this unicode character
        if first_byte <= 0b01111111 {
            advance_bytes = 1;
        } else if first_byte <= 0b11011111 {
            advance_bytes = 2;
        } else if first_byte <= 0b11101111 {
            advance_bytes = 3;
        } else {
            advance_bytes = 4;
        }

        characters_count += 1;
        *i += advance_bytes;
    }
}

enum Error {
    InvalidResponse,
    ExpectedEOF,
    ExpectedEntry,
    // ExpectedFoodItem, 
    // ExpectedCalorieValue,
    // InvalidQuantity,
    // InvalidCaloriesMissingKcal,
    // InvalidCalories,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidResponse => {
                write!(f, "Invalid response")
            },
            Error::ExpectedEOF => {
                write!(f, "Expected end of file")
            },
            Error::ExpectedEntry => {
                write!(f, "Expected entry")
            },
        }
    }
}

struct Parser {
    text: String,
    end_index: usize,
    i: usize,
    entries: Vec<EntryEntity>,
}

struct EntryEntity {
    date: String,
    sections: Vec<EntrySectionEntity>,
}

struct EntrySectionEntity {
    id: String,
    items: Vec<Item>,
}

struct Item {
    title: String,
    quantity: f32,
    measurement: QuantityMeasurement,
    calories: f32,
}

enum QuantityMeasurement {
    Portion,
    Liter,
    Kilogram,
    Cup,
}

// done

fn make_http_request() -> Result<String, Error> {
    Ok(minreq::get("https://whoniverse-app.com/calcal/main.php")
        .send()
        .map_err(|_e| { Error::InvalidResponse })?
        .as_str()
        .map_err(|_e| { Error::ExpectedEOF })?
        .to_owned())
}

// unit tests

fn test_advance_if_possible_after_unicode() {
    let string = "üabc";
    let bytes = string.as_bytes();

    let mut i = 0;
    let after = 2;
    let expected = 4;
    
    advance_if_possible_after_unicode_s(bytes, &mut i, bytes.len(), after);
    
    let test = i == expected;
    if test {
        println!("Test 1: OK");
    } else {
        println!("Test 1: FAIL! i: {}, expected {}", i, expected);
    }
}

fn test_next_matches_ascii() {
    let string1 = "mensch";
    let test1 = next_matches_ascii_s(string1.as_bytes(), 0, string1.len(), "mensch");

    let string2 = "übermensch";
    let test2 = next_matches_ascii_s(string2.as_bytes(), 0, string2.len(), "mensch");
    
    if test1 && !test2 {
        println!("Test 2: OK");
    } else {
        println!("Test 2: FAIL!");
    }
}

fn test_first_index() {
    let string = "übermensch bin ich";
    let index = first_index_s(string.as_bytes(), 0, string.len(), ' ');

    let expected = 11;
    let test = index == expected;
    if test {
        println!("Test 3: OK");
    } else {
        println!("Test 3: FAIL! i: {}, expected {}", index, expected);
    }
}

const NOT_FOUND: i32 = -1;

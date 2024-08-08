#![allow(dead_code)] // todo: remove
#![allow(unused_imports)] // todo: remove

use std::process::exit;
use std::fmt;

fn main() {
    test_advance_if_possible_after_unicode();
    println!("---- start program ------");

    println!("Hello world!\n");

    let response = make_http_request().unwrap_or_else(|error| {
        eprintln!("An error occured while making http request: {error}");
        exit(1)
    });

    parse_entities(response).unwrap_or_else(|error| {
        eprintln!("An error occured while making http request: {error}");
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

    println!("first tokens: \n{}", parser.text[parser.i..parser.i + 50].to_string());

    while parser.i < parser.text.len() {
            eat_whitespaces_and_newlines(&mut parser);

            if !next_matches_ascii(&parser, "Date: ") {
                // print_error_position();
                return Err(Error::ExpectedEntry)
            }


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


    return Err(Error::InvalidResponse)
}

fn next_matches_ascii(parser: &Parser, search: &str) -> bool {
    let remaining_length = parser.end_index - parser.i;

    if remaining_length < search.len() {
        return false
    }

    // [parser.i..parser.i + search.len()]

    let byte_slice = parser.text.as_bytes();
    return byte_slice == search.as_bytes()
}

fn eat_whitespaces_but_no_newlines(parser: &mut Parser) {
    while parser.end_index > parser.i && is_whitespace_but_not_newline(parser) {
        let i_value = parser.i.clone(); // bad code, how to write it inline?
        advance_if_possible_after_unicode(parser.text.as_bytes(), &mut parser.i, parser.end_index, i_value);
    }
}

fn eat_whitespaces_and_newlines(parser: &mut Parser) {
    while parser.end_index > parser.i && is_whitespace(parser) {
        let i_value = parser.i.clone();
        advance_if_possible_after_unicode(parser.text.as_bytes(), &mut parser.i, parser.end_index, i_value);
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

fn advance_if_possible_after_unicode(text: &[u8], i: &mut usize, end_index: usize, after: usize) {
    if end_index < after {
        *i = end_index;
        return
    }

    *i += 1;
    let mut count = 1;

    while *i < end_index {
        if count == after {
            break
        }

        let first_byte = text[*i];
        let advance_i_by: usize;
        
        // figure out how many bytes long is this unicode character
        if first_byte <= 0b01111111 {
            advance_i_by = 1;
        } else if first_byte <= 0b11011111 {
            advance_i_by = 2;
        } else if first_byte <= 0b11101111 {
            advance_i_by = 3;
        } else {
            advance_i_by = 4;
        }

        count += 1;
        *i += advance_i_by;
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
    let string = "über-человек";
    let bytes = string.as_bytes();
    let mut i = 0;

    let diff: usize = 10;
    let expected = 16;
    
    advance_if_possible_after_unicode(bytes, &mut i, bytes.len(), diff);
    
    let test = i == expected;
    if test {
        println!("Test 1: OK");
    } else {
        println!("Test 1: FAIL! i: {}, expected {}", i, expected);
    }
}


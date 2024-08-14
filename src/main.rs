#![allow(dead_code)] // todo: remove
#![allow(unused_imports)] // todo: remove
#![allow(unused_variables)] // todo: remove

use std::process::exit;
use std::fmt;
use QuantityMeasurement::*;

fn main() {
    test_advance_if_possible_after_unicode();
    test_next_matches_ascii();
    test_first_index();
    test_eat_whitespaces_but_not_newlines();
    test_get_quantity();

    println!("---- start program ------");

    println!("Hello world!\n");

    let response_string = make_http_request().unwrap_or_else(|error| {
        eprintln!("An error occured while making http request: {error}");
        exit(1);
    });


    parse_entities(response_string).unwrap_or_else(|error| {
        eprintln!("An error occured while parsing response: {error}");
        exit(1);
    });
}

// todo: make sure to trim whitespaces or  with newlines carefully!
fn parse_entities(string: String) -> Result<Vec<EntryEntity>, Error> {
    let mut parser = Parser {
        i: 0,
        end_index: string.len(),
        text: string,
        entries: vec![],
    };

    while parser.end_index > parser.i {
        eat_whitespaces_and_newlines(&mut parser);

        let index_after_date = first_index(&parser, ' ');
        let next_is_date_declaration = next_matches_ascii(&parser, "Date: ");
        if !next_is_date_declaration || index_after_date == NOT_FOUND {
            println!("next_matches_ascii date: {}; index_after_date: {}", next_is_date_declaration, index_after_date);
            print_error_position(&parser);
            return Err(Error::ExpectedEntry);
        }

        advance_if_possible_after_unicode(&mut parser, index_after_date as usize);

        eat_whitespaces_but_not_newlines(&mut parser);
        let date_newline_index = first_index(&parser, '\n');
        if date_newline_index == NOT_FOUND {
            println!("next: {}", &parser.text[parser.i..parser.i+10]);
            println!("date_newline_index: {}", date_newline_index);
            print_error_position(&parser);
            return Err(Error::ExpectedEOF);
        }

        let date_string = substring_with_length(&parser, date_newline_index as usize - 1).trim().to_string();

        advance_if_possible_after_unicode(&mut parser, date_newline_index as usize);

        let mut sections: Vec<EntrySectionEntity> = vec!();

        while parser.end_index > parser.i {
            eat_whitespaces_but_not_newlines(&mut parser);

            // Date means new entry
            let next_is_another_date_declaration = next_matches_ascii(&parser, "Date: ");
            if next_is_another_date_declaration {
                println!("breaking @section-1");
                print_error_position(&parser);
                break;
            }

            let section_separator_index = first_index(&parser, '-');
            if section_separator_index == NOT_FOUND {
                if sections.len() == 0 {
                    eat_whitespaces_but_not_newlines(&mut parser);
                    let i = parser.i.clone();
                    advance_if_possible_after_unicode(&mut parser, i);
                    println!("breaking @section-2");
                    print_error_position(&parser);
                    continue;
                }

                println!("breaking @section-3");
                print_error_position(&parser);
                break;
            }

            let section_name = substring_with_length(&parser, section_separator_index as usize - 1).trim().to_string();

            advance_if_possible_after_unicode(&mut parser, section_separator_index as usize);

            let section_newline_index = first_index(&parser, '\n');
            if section_newline_index == NOT_FOUND {
                print_error_position(&parser);
                return Err(Error::ExpectedEOF);
            }

            advance_if_possible_after_unicode(&mut parser, section_newline_index as usize);

            let mut food_items: Vec<Item> = vec!();

            while parser.end_index > parser.i {
                eat_whitespaces_but_not_newlines(&mut parser);

                // new line means end of section
                if next_matches_ascii(&parser, "\n") {
                    let i = parser.i.clone();
                    advance_if_possible_after_unicode(&mut parser, 0);
                    break;
                }

                let item_start_index = first_index(&parser, '-');
                if item_start_index == NOT_FOUND {
                    if food_items.len() == 0 {
                        print_error_position(&parser);
                        return Err(Error::ExpectedFoodItem);
                    }
                    println!("breaking @food-2");
                    print_error_position(&parser);
                    break;
                }

                advance_if_possible_after_unicode(&mut parser, item_start_index as usize);
                eat_whitespaces_but_not_newlines(&mut parser);

                let item_name_separator = first_index(&parser, ',');
                if item_name_separator == NOT_FOUND {
                    print_error_position(&parser);
                    return Err(Error::ExpectedCalorieValue);
                }

                let item_name = substring_with_length(&parser, item_name_separator as usize - 1).trim().to_string();
                advance_if_possible_after_unicode(&mut parser, item_name_separator as usize);

                let quantity_value: f32;
                let measurement: QuantityMeasurement;
                let mut item_end_of_line = first_index(&parser, '\n');
                if item_end_of_line == NOT_FOUND {
                    item_end_of_line = parser.end_index as i32; // todo: do we care about it here?
                }
                let commas_count = count_characters_in_string(&parser.text[parser.i..parser.i + item_end_of_line as usize], ',');

                if commas_count > 0 { // optionally parse quantity
                    eat_whitespaces_but_not_newlines(&mut parser);

                    let item_quantity_separator = first_index(&parser, ',');
                    if item_quantity_separator == NOT_FOUND {
                        print_error_position(&parser);
                        return Err(Error::ExpectedCalorieValue);
                    }

                    let item_quantity_string = substring_with_length(&parser, item_quantity_separator as usize - 1).trim(); // whitespaces

                    // finalise item
                    if let Some(quantity_tuple) = get_quantity(item_quantity_string) {
                        quantity_value = quantity_tuple.0;
                        measurement = quantity_tuple.1;
                    } else {
                        print_error_position(&parser);
                        return Err(Error::InvalidQuantity);
                    }

                    advance_if_possible_after_unicode(&mut parser, item_quantity_separator as usize);
                } else {
                    quantity_value = 1.0;
                    measurement = Portion;
                }

                eat_whitespaces_but_not_newlines(&mut parser);

                let mut item_newline = first_index(&parser, '\n');
                if item_newline == NOT_FOUND {
                    // todo: possibly bug, we will pass end_index as size below
                    item_newline = parser.end_index as i32; // todo: do we care about i32 here?
                }
                let mut item_calorie_string = substring_with_length(&parser, item_newline as usize - 1).trim(); // whitespaces

                if !item_calorie_string.contains(" kcal") {
                    print_error_position(&parser);
                    return Err(Error::InvalidCaloriesMissingKcal);
                }

                let len_without_suffix = item_calorie_string.len() - " kcal".len();
                item_calorie_string = &item_calorie_string[0..len_without_suffix];

                let calories_value = if let Ok(value) = item_calorie_string.parse::<f32>() {
                    value
                } else {
                    print_error_position(&parser);
                    return Err(Error::InvalidCalories);
                };
                advance_if_possible_after_unicode(&mut parser, item_newline as usize);

                let food_item = Item { 
                    title: item_name,
                    quantity: quantity_value, 
                    measurement: measurement, 
                    calories: calories_value 
                };

                let a = &food_item.title;
                let b = &food_item.quantity;
                let c = &food_item.measurement;
                let d = &food_item.calories;
                println!("* Item added: '{}' '{} ({})', '{}'", food_item.title, food_item.quantity, food_item.measurement, food_item.calories);
                food_items.push(food_item);
            }

            let section = EntrySectionEntity {
                id: section_name.to_string(),
                items: food_items
            };
            println!(":: Section added: {}", section.id);
            sections.push(section);

            eat_whitespaces_but_not_newlines(&mut parser);

            let remaining_text = substring_with_length(&parser, parser.end_index - parser.i).trim(); // whitespaces
            if next_matches_ascii(&parser, "Total: ") {
                // println!("breaking @section-4");

                let newline_after_total = first_index(&parser, '\n');
                if newline_after_total == NOT_FOUND {
                    let end_index = parser.end_index;
                    advance_if_possible_after_unicode(&mut parser, end_index);
                } else {
                    advance_if_possible_after_unicode(&mut parser, newline_after_total as usize);
                    eat_whitespaces_but_not_newlines(&mut parser);

                    let current_i = parser.i.clone();
                    advance_if_possible_after_unicode(&mut parser, 0);
                }
                break;
            } 
        }

        let entry = EntryEntity {
            date: date_string,
            sections: sections
        };
        println!("=> Entry added: {}\n", entry.date);
        parser.entries.push(entry);

        eat_whitespaces_and_newlines(&mut parser);
    }

    println!("Done! Added {} entrie(s)", parser.entries.len());
    return Ok(parser.entries);
}

fn count_characters_in_string(string: &str, search: char) -> usize {
    let string_length = string.len();
    let mut count = 0;
    let mut i = 0;
    while string_length > i { 
        if string.as_bytes()[i] as char == search {
            count += 1;
        }
        advance_if_possible_after_unicode_s(string.as_bytes(), &mut i, string_length, 0);
    }
    return count;
}

// accepts byte offset, returns character offset!
fn first_index(parser: &Parser, search: char) -> i32 {
    first_index_s(parser.text.as_bytes(), parser.i, parser.end_index, search)
}
fn first_index_s(bytes: &[u8], offset: usize, end_index: usize, search: char) -> i32 {
    let mut i = offset;
    let mut char_i = 0;
    while end_index > i {
        if bytes[i] as char == search {
            // return (i - offset) as i32;
            return char_i;
        }
        char_i += 1;
        advance_if_possible_after_unicode_s(bytes, &mut i, end_index, 0);
    }
    return NOT_FOUND;
}

fn next_matches_ascii(parser: &Parser, search: &str) -> bool {
    next_matches_ascii_s(parser.text.as_bytes(), parser.i, parser.end_index, search)
}
fn next_matches_ascii_s(bytes: &[u8], i: usize, end_index: usize, search: &str) -> bool {
    let remaining_length = end_index - i;

    let search_length: usize = search.len();
    if remaining_length < search_length {
        return false;
    }

    let byte_slice = &bytes[i..i + search_length];
    return byte_slice == search.as_bytes();
}

fn print_error_position(parser: &Parser) {
    let trail_start = std::cmp::max(parser.i-10, 0);
    let previous_symbols = &parser.text[trail_start..parser.i];
    let count_of_newlines = count_characters_in_string(previous_symbols, '\n');

    let next_symbols = &parser.text[parser.i..std::cmp::min(parser.i + 50, parser.end_index)];
    let message_string = "position: ...";
    println!("{}{}\x1b[91m{}\x1b[0m...", message_string, previous_symbols.replace("\n", "\\n"), next_symbols.replace("\n", "\\n"));

    // todo: respect unicode?
    let cursor_offset = parser.i - trail_start + count_of_newlines + message_string.len();
    for i in 0..cursor_offset {
        print!(" ")
    }
    println!("^ is the next character at the cursor\n");
}

fn eat_whitespaces_but_not_newlines(parser: &mut Parser) {
    while parser.end_index > parser.i && parser.text.as_bytes()[parser.i] as char == ' ' {
        advance_if_possible_after_unicode(parser, 0);
    }
}

fn eat_whitespaces_and_newlines(parser: &mut Parser) {
    while parser.end_index > parser.i && is_whitespace(parser) {
        let i_value = parser.i.clone();
        advance_if_possible_after_unicode(parser, i_value);
    }
}

fn is_whitespace(parser: &Parser) -> bool {
    let c = parser.text.as_bytes()[parser.i] as char;
    return c == ' ' || c == '\n';
}

fn substring_with_length(parser: &Parser, length: usize) -> &str {
    let mut i = parser.i;
    advance_if_possible_after_unicode_s(parser.text.as_bytes(), &mut i, parser.end_index, length);
    return std::str::from_utf8(&parser.text.as_bytes()[parser.i..i]).unwrap(); // todo: unsafe
}

fn advance_if_possible_after_unicode(parser: &mut Parser, after: usize) {
    advance_if_possible_after_unicode_s(parser.text.as_bytes(), &mut parser.i, parser.end_index, after);
}
fn advance_if_possible_after_unicode_s(text: &[u8], i: &mut usize, end_index: usize, after: usize) {
    let initial_index = i.clone();

    if end_index < after {
        *i = end_index;
        return;
    }

    let mut characters_count = 0;

    while *i < end_index {
        if characters_count > after {
            break;
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

    // println!("advancing through: {}", std::str::from_utf8(&text[initial_index..*i]).unwrap());
}

enum Error {
    InvalidResponse,
    ExpectedEOF,
    ExpectedEntry,
    ExpectedFoodItem, 
    ExpectedCalorieValue,
    InvalidQuantity,
    InvalidCaloriesMissingKcal,
    InvalidCalories,
}

impl fmt::Display for QuantityMeasurement{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Portion => {
                write!(f, "PORTION")
            },
            Liter => {
                write!(f, "L")
            },
            Kilogram => {
                write!(f, "KG")
            },
            Cup => {
                write!(f, "CUP")
            },
        }
    }
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
            Error::ExpectedFoodItem => {
                write!(f, "Expected food item")
            },
            Error::ExpectedCalorieValue => {
                write!(f, "Expected calorie value")
            },
            Error::InvalidQuantity => {
                write!(f, "Invalid quantity")
            },
            Error::InvalidCaloriesMissingKcal => {
                write!(f, "Expected calorie value with kcal")
            },
            Error::InvalidCalories => {
                write!(f, "Expected correct calories value")
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

#[derive(PartialEq)]
enum QuantityMeasurement {
    Portion,
    Liter,
    Kilogram,
    Cup,
}

fn make_http_request() -> Result<String, Error> {
    Ok(minreq::get("https://whoniverse-app.com/calcal/main.php")
        .send()
        .map_err(|_e| { Error::InvalidResponse })?
        .as_str()
        .map_err(|_e| { Error::ExpectedEOF })?
        .to_owned())
}

fn get_quantity(text: &str) -> Option<(f32, QuantityMeasurement)> {
    assert!(text.is_ascii(), "Quantity text must always be ascii!");

    if let Ok(value) = text.parse::<f32>() {
        return Some((value, Portion));
    }

    for measurement in [Portion, Liter, Kilogram, Cup] {
        let acceptable_values: Vec<&str> = match measurement {
            Liter => ["milliliter", "millilitre", "liter", "litre", "ml", "l"].to_vec(),
            Kilogram => ["kilogram", "gram", "kg", "gr", "g"].to_vec(),
            Cup => ["cup"].to_vec(),
            Portion => ["portion", "part"].to_vec(),
        };

        let subdivision_values = [ "gram", "gr", "g", "milliliter", "millilitre", "ml" ];

        'inner: for value in acceptable_values {
            if !text.ends_with(value) {
                continue 'inner;
            }

            let len_without_suffix = text.len() - value.len();
            let text_without_suffix = &text[0..len_without_suffix].trim();

            let quantity_value: f32;
            match text_without_suffix.parse::<f32>() {
                Ok(value) => { quantity_value = value; }
                Err(_) => { return None; }
            }

            let subdivision: f32 = if subdivision_values.contains(&value) { 1000.0 } else { 1.0 };
            return Some((quantity_value / subdivision, measurement));
        }
    }
    return None;
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
        println!("Test 2: FAIL! 1: {test1}; 2: {test2}");
    }
}

fn test_first_index() {
    let string = "übermensch bin ich";

    let index1 = first_index_s(string.as_bytes(), 0, string.len(), ' ');
    let expected1 = 10;
    let test1 = index1 == expected1;

    let index2 = first_index_s(string.as_bytes(), 5, string.len(), ' ');
    let expected2 = 10 - 4;
    let test2 = index2 == expected2;

    if test1 && test2 {
        println!("Test 3: OK");
    } else {
        println!("Test 3: FAIL! 1: {test1}; 2: {test2}");
    }
}

fn test_eat_whitespaces_but_not_newlines() {
    let string = "        \naedfawd\n ".to_string();

    let mut parser = Parser {
        i: 0,
        end_index: string.len(),
        text: string,
        entries: vec![],
    };

    eat_whitespaces_but_not_newlines(&mut parser);

    let expected = 8;
    let test = parser.i == expected;
    if test {
        println!("Test 4: OK");
    } else {
        println!("Test 4: FAIL! i: {}, expected {}", parser.i, expected);
    }
}

fn test_get_quantity() {
    let test1 = if let Some(value) = get_quantity("1500 ml") {
        value.0 == 1.5 && value.1 == Liter
    } else { false };

    let test2 = if let Some(value) = get_quantity("2") {
        value.0 == 2.0 && value.1 == Portion
    } else { false };

    if test1 && test2 {
        println!("Test 5: OK");
    } else {
        println!("Test 5: FAIL! 1: {test1}; 2: {test2}");
    }
}

const NOT_FOUND: i32 = -1;

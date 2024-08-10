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
    println!("---- start program ------");

    println!("Hello world!\n");

    let response = make_http_request().unwrap_or_else(|error| {
        eprintln!("An error occured while making http request: {error}");
        exit(1);
    });

    parse_entities(response).unwrap_or_else(|error| {
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

    while parser.i < parser.text.len() {
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

        let date_string = substring_with_length(&parser, date_newline_index as usize).trim();
        println!("Found entry: '{}'", date_string);

        advance_if_possible_after_unicode(&mut parser, date_newline_index as usize);

        let mut sections: Vec<EntrySectionEntity> = vec!();

        while parser.end_index > parser.i {
            eat_whitespaces_but_not_newlines(&mut parser);

            // Date means new entry
            let next_is_another_date_declaration = next_matches_ascii(&parser, "Date: ");
            if next_is_another_date_declaration {
                break;
            }

            let section_separator_index = first_index(&parser, '-');
            if section_separator_index == NOT_FOUND {
                if sections.len() == 0 {
                    eat_whitespaces_but_not_newlines(&mut parser);
                    let i = parser.i.clone();
                    advance_if_possible_after_unicode(&mut parser, i);
                    continue;
                }
                break;
            }

            let section_name = substring_with_length(&parser, section_separator_index as usize).trim();
            println!("Found section: '{}'", section_name);
            
            advance_if_possible_after_unicode(&mut parser, section_separator_index as usize);

            let section_newline_index = first_index(&parser, '\n');
            if section_newline_index == NOT_FOUND {
                print_error_position(&parser);
                return Err(Error::ExpectedEOF);
            }
            
            advance_if_possible_after_unicode(&mut parser, section_newline_index as usize);
            println!("-----\n{}\n-----", &parser.text[parser.i..parser.i+20]);

            let mut food_items: Vec<Item> = vec!();

            while parser.end_index > parser.i {
                eat_whitespaces_and_newlines(&mut parser);

                // new line means end of section
                if next_matches_ascii(&parser, "\n") {
                    let i = parser.i.clone();
                    advance_if_possible_after_unicode(&mut parser, i);
                    break;
                }

                let item_start_index = first_index(&parser, '-');
                if item_start_index == NOT_FOUND {
                    if food_items.len() == 0 {
                        print_error_position(&parser);
                        return Err(Error::ExpectedFoodItem);
                    }
                    break;
                }

                advance_if_possible_after_unicode(&mut parser, item_start_index as usize);
                eat_whitespaces_but_not_newlines(&mut parser);

                let item_name_separator = first_index(&parser, ',');
                if item_name_separator == NOT_FOUND {
                    print_error_position(&parser);
                    return Err(Error::ExpectedCalorieValue);
                }

                let item_name = substring_with_length(&parser, item_name_separator as usize).trim();
                println!("Found item: {}", item_name);
                advance_if_possible_after_unicode(&mut parser, item_name_separator as usize);

                let quantity_value: f32;
                let measurement: QuantityMeasurement;
                let item_end_of_line = first_index(&parser, '\n');
                let commas_count = count_characters_in_string(&parser.text[parser.i..parser.i + item_end_of_line as usize], ',');
                println!("item_end_of_line: {}, commas_count: {}", item_end_of_line, commas_count);

                if commas_count > 0 { // optionally parse quantity
                    eat_whitespaces_but_not_newlines(&mut parser);

                    let item_quantity_separator = first_index(&parser, ',');
                    if item_quantity_separator == NOT_FOUND {
                        print_error_position(&parser);
                        return Err(Error::ExpectedCalorieValue);
                    }

                    let item_quantity_string = substring_with_length(&parser, item_quantity_separator as usize).trim(); // whitespaces
                    let quantityTuple = get_quantity(item_quantity_string);
                    advance_if_possible_after_unicode(&mut parser, item_quantity_separator as usize);



                    

                } else {
                    quantity_value = 1.0;
                    measurement = Portion;
                }



//     let quantityValue: Float, measurement: EntryEntity.QuantityMeasurement
//     let itemEndOfLine = textRemainder.firstIndex(of: "\n") ?? endIndex
//     let commasCount = textRemainder[i..<itemEndOfLine].filter { $0 == "," }.count
//     if commasCount > 0 { // optionally parse quantity
//         eatWhitespaces()
//         guard let itemQuantitySeparator = textRemainder.firstIndex(of: ",") else {
//             printErrorPosition()
//             throw Error.expectedCalorieValue
//         }
//         let itemQuantityString = String(textRemainder[i..<itemQuantitySeparator]).trimmingCharacters(in: .whitespaces)
//         advanceIfPossible(after: itemQuantitySeparator)
        
//         // finalise item
//         guard let (_quantityValue, _measurement) = Self.getQuantity(text: itemQuantityString)
//         else {
//             printErrorPosition()
//             throw Error.invalidQuantity
//         }
//         quantityValue = _quantityValue
//         measurement = _measurement
//     } else {
//         quantityValue = 1
//         measurement = .portion
//     }
    
//     eatWhitespaces()
//     let itemNewLine = textRemainder.firstIndex(of: "\n") ?? endIndex
//     var itemCalorieString = String(textRemainder[i..<itemNewLine]).trimmingCharacters(in: .whitespaces)
//     guard itemCalorieString.contains(" kcal") else {
//         printErrorPosition()
//         throw Error.invalidCaloriesMissingKcal
//     }
//     itemCalorieString = String(itemCalorieString.dropLast(" kcal".count))
//     advanceIfPossible(after: itemNewLine)
    
//     guard let caloriesValue = itemCalorieString.floatValue else {
//         printErrorPosition()
//         throw Error.invalidCalories
//     }
    
//     let foodItem = EntryEntity.Item(
//         title: itemName,
//         quantity: quantityValue,
//         measurement: measurement,
//         calories: caloriesValue
//     )
//     foodItems.append(foodItem)
            }

        }



        // for debugging
        parser.i += 1;
    }


    todo!()
}

fn count_characters_in_string(string: &str, search: char) -> usize {
    let string_length = string.len();
    let mut count = 0;
    let mut i = 0;
    while string_length > i { 
        if string.as_bytes()[i] as char == ' ' {
            count += 1;
        }
        advance_if_possible_after_unicode_s(string.as_bytes(), &mut i, string_length, 0);
    }
    return count;
}

fn first_index(parser: &Parser, search: char) -> i32 {
    first_index_s(parser.text.as_bytes(), parser.i, parser.end_index, search)
}
fn first_index_s(bytes: &[u8], offset: usize, end_index: usize, search: char) -> i32 {
    let mut i = offset;
    while end_index > i {
        if bytes[i] as char == search {
            return (i - offset) as i32;
        }
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
    let previous_symbols = &parser.text[parser.i..std::cmp::min(parser.i + 100, parser.end_index)];
    println!("Parser: Error occured right before this text:\n{}", previous_symbols);
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
    return std::str::from_utf8(&parser.text.as_bytes()[parser.i..i]).unwrap();
}

fn advance_if_possible_after_unicode(parser: &mut Parser, after: usize) {
    advance_if_possible_after_unicode_s(parser.text.as_bytes(), &mut parser.i, parser.end_index, after);
}
fn advance_if_possible_after_unicode_s(text: &[u8], i: &mut usize, end_index: usize, after: usize) {
    if end_index < after {
        *i = end_index;
        return;
    }

    let mut characters_count = 0;

    while *i < end_index {
        // println!(" -- '{}'", text[*i] as char);
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
}

enum Error {
    InvalidResponse,
    ExpectedEOF,
    ExpectedEntry,
    ExpectedFoodItem, 
    ExpectedCalorieValue,
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
            Error::ExpectedFoodItem => {
                write!(f, "Expected food item")
            },
            Error::ExpectedCalorieValue => {
                write!(f, "Expected calorie value")
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

fn quantity_measurement_all_cases() -> [QuantityMeasurement; 4] {
    [Portion, Liter, Kilogram, Cup]
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

fn get_quantity(text: &str) -> Option<(f32, QuantityMeasurement)> {
    if let Ok(value) = text.parse::<f32>() {
        return Some((value, Portion));
    }

    todo!()
}

// static func getQuantity(text: String) -> (Float, EntryEntity.QuantityMeasurement)? {
//     if let quantityValue = text.floatValue {
//         return (quantityValue, .portion)
//     }
    
//     for measurement in EntryEntity.QuantityMeasurement.allCases {
        
//         let acceptableValues = switch measurement {
//         case .liter: ["milliliter", "millilitre", "liter", "litre", "ml", "l"]
//         case .kilogram: ["kilogram", "gram", "kg", "gr", "g"]
//         case .cup: ["cup"]
//         case .portion: ["portion", "part"]
//         }
        
//         let subdivisionValues = [
//             "gram", "gr", "g", "milliliter", "millilitre", "ml"
//         ]
        
//         for value in acceptableValues {
//             guard text.hasSuffix(value) else { continue }
//             let textWithoutSuffix = String(text.dropLast(value.count)).trimmingCharacters(in: .whitespaces)
//             guard let quantityValue = textWithoutSuffix.floatValue else { return nil }
//             let subdivision: Float = subdivisionValues.contains(value) ? 1000.0 : 1
//             return (quantityValue / subdivision, measurement)
//         }
//     }
//     return nil
// }

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

    let index1 = first_index_s(string.as_bytes(), 0, string.len(), ' ');
    let expected1 = 11;
    let test1 = index1 == expected1;

    let index2 = first_index_s(string.as_bytes(), 5, string.len(), ' ');
    let expected2 = 11 - 5;
    let test2 = index2 == expected2;

    if test1 && test2 {
        println!("Test 3: OK");
    } else {
        println!("Test 3: FAIL!");
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

const NOT_FOUND: i32 = -1;

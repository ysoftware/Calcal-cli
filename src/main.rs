use std::process::exit;
use std::fmt;

fn main() {
    println!("Hello world!\n");

    let response = make_http_request().unwrap_or_else(|error| {
        eprintln!("An error occured while making http request: {error}");
        exit(1)
    });

    parse_entities(response);
}

fn make_http_request() -> Result<String, Error> {
   let string_value = minreq::get("https://whoniverse-app.com/calcal/main.php")
        .send()
        .map_err(|_e| {
            Error::InvalidResponse
        })?
        .as_str()
        .map_err(|_e| {
            Error::ExpectedEOF
        })?
        .to_owned();

    Ok(string_value)
}

fn parse_entities(string: String) {
    let mut i: usize = 0;

    while i < string.len() {
        i += 1;
    }

    println!("last token: {}", string.as_bytes()[i-1]);
}

enum Error {
    InvalidResponse, // (i32),
    ExpectedEOF,
    // ExpectedEntry,
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
                write!(f, "Expected End of file")
            }
        }
    }
}

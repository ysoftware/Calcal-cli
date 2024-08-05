fn main() {
    println!("Hello world!\n");
    let _response = make_http_request();
    parse_entities();
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

fn parse_entities() {
   todo!() 
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

use minreq;

fn main() {
    println!("Hello world!\n");
    let response = make_http_request();
}

fn make_http_request() -> Result<String, Error> {
    let response = minreq::get("https://whoniverse-app.com/calcal/main.php")
        .send().map_err(|_e| {
            Error::InvalidResponse(0)
        })?
        .as_str().map_err(|_e| {
            Error::ExpectedEOF
        });

    response
}

fn parse_entities() {
    
}

enum Error {
    InvalidResponse(i32),
    ExpectedEntry,
    ExpectedEOF,
    ExpectedFoodItem, 
    ExpectedCalorieValue,
    InvalidQuantity,
    InvalidCaloriesMissingKcal,
    InvalidCalories,
}

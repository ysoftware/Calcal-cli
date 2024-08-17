mod parser;
mod terminal;

use Color::*;

enum State { List, Input, Calendar }

struct App {
    state: State,
    entries: Vec<parser::EntryEntity>,
    width: usize,
    height: usize,
    selected_entry_index: usize,
    input_buffer: String,
}

fn main() {
    terminal::prepare_terminal();
    terminal::clear_window();
    parser::test_all();

    println!("Starting download...");
    let response_string = make_http_request().unwrap_or_else(|error| {
        eprintln!("An error occured while making http request: {error}");
        std::process::exit(1);
    });

    let entries = parser::parse_entities(response_string).unwrap_or_else(|error| {
        eprintln!("An error occured while parsing response: {error}");
        std::process::exit(1);
    });

    let entries_count = entries.len() - 1; // todo: will it crash if entires are empty?

    let mut app = App {
        state: State::List,
        entries: entries,
        width: 0,
        height: 0,
        selected_entry_index: entries_count,
        input_buffer: "".to_string(),
    };

    loop {
        if process_input(&mut app) {
            draw(&app);
        } else {
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
    }

    terminal::restore_terminal();
}

fn process_window_resize(app: &mut App) -> bool {
    let (new_width, new_height) = terminal::get_window_size();
    let did_resize_window = new_width != app.width || new_height != app.height;
    app.width = new_width;
    app.height = new_height;
    return did_resize_window;
}

fn process_input(app: &mut App) -> bool {
    let did_resize_window = process_window_resize(app);

    let did_process_input: bool = match app.state {
        State::List => { process_input_list(app) },
        State::Input => { process_input_input(app) },
        State::Calendar => { todo!() },
    };
    return did_resize_window || did_process_input;
}

fn draw(app: &App) {
    terminal::clear_window();

    // println!("{}", app.input_buffer);

    match app.state {
        State::List => { draw_list(&app); },
        State::Input => { draw_input(&app); },
        State::Calendar => { draw_calendar(&app); },
    }
}

// SPECIFIC VIEWS

fn process_input_list(app: &mut App) -> bool {
    let mut did_process_input = false;
    if let Some(input) = terminal::get_input() { // todo: unicode
        app.input_buffer.push(input);
        did_process_input = true; // this should be inside of blocks to redraw only when needed
                                  // but it is leading to slow scrolling through pages

        if input == '\n' {
            app.input_buffer = "".to_string();
        }
        else if input as usize == 68 { // arrow left
            if app.selected_entry_index - 1 > 0 {
                app.selected_entry_index -= 1;
            }
        }
        else if input as usize == 67 { // arrow right
            if app.selected_entry_index + 1 < app.entries.len() {
                app.selected_entry_index += 1;
            }
        }
        else if input == 'n' {
            app.state = State::Input;
        }
    }

    return did_process_input;
}

fn draw_list(app: &App) {
    if app.height > 40 && app.width > 50 { draw_empty(); }

    let selected_entry = &app.entries[app.selected_entry_index];
    let mut entry_calories = 0.0;

    for section in &selected_entry.sections {
        for item in &section.items {
            entry_calories += item.calories;
        }
    }

    draw_line_right(
        format!("{}", selected_entry.date), BlackBg,
        format!("{} kcal", entry_calories), BlueBrightBg,
        app.width, 0
    );

    for section in &selected_entry.sections {
        let mut section_calories = 0.0;
        for item in &section.items {
            section_calories += item.calories;
        }

        draw_empty();
        draw_line_right(
            format!("{}", section.id), BlueBright,
            format!("{} kcal", section_calories), BlueBright,
            app.width, 
            20 // align right text in the middle of the line
        );

        for i in 0..section.items.len() {
            let item = &section.items[i];
            let left_color = if i % 2 == 1 { White } else { BlackBg };
            let right_color = if i % 2 == 1 { White } else { BlackBg };

            draw_line_right(
                format!("- {}, {} {}", item.title, item.quantity, item.measurement), left_color,
                format!("{} kcal", item.calories), right_color,
                app.width, 0
            );
        }
    }
}

fn process_input_input(app: &mut App) -> bool {
    let mut did_process_input = false;
    if let Some(input) = terminal::get_input() { // todo: unicode
        app.state = State::List;
    }
    return did_process_input;
}

fn draw_input(app: &App) {
    println!("Inputting shit");
}

fn process_input_calendar(app: &mut App) -> bool {
    return false;
}

fn draw_calendar(app: &App) {

}

// DRAW TEXT

fn draw_empty() {
    println!("");
}

fn draw_line_right(string_left: String, color_left: Color, string_right: String, color_right: Color, width: usize, left_limit: usize) {
    let draw_width = std::cmp::min(width, 50);

    let empty_width = width - draw_width;
    let left_side_padding = if empty_width > 10 { empty_width / 2 } else { 0 };
    for _ in 0..left_side_padding { print!(" "); }

    let length_left = string_left.chars().count();
    let length_right = string_right.chars().count();
    let padding = ".. ";

    if length_left + length_right + padding.len() <= draw_width {
        print!("{}{}", color_start(color_left), string_left);

        let mut spacing = draw_width - (length_left + length_right);
        if left_limit > 0 && spacing > 1 { // todo: spacing > 1? >2?
            spacing = std::cmp::min(spacing, left_limit - (length_left));
        }
        for _ in 0..spacing { print!(" "); }

        println!("{}{}{}", color_start(color_right), string_right, color_end);
    } else {
        if length_right <= draw_width {
            if length_right + padding.len() < draw_width {
                let rest_width = draw_width - length_right - padding.len();
                let truncated_string = truncate(string_left, rest_width);
                print!("{}{}{}{}", color_start(color_left), truncated_string, padding, color_end);
            }

            println!("{}{}{}", color_start(color_right), string_right, color_end);
        } else {
            let truncated_string = truncate(string_right, draw_width);
            println!("{}{}{}", color_start(color_right), truncated_string, color_end);
        }
    }
}

fn truncate(s: String, n: usize) -> String {
    let n = s.len().min(n);
    if let Some(m) = (0..=n).rfind(|m| s.is_char_boundary(*m)) {
        return s[..m].to_string();
    } else {
        println!("Unable to truncate string \"{}\" by {} characters.", s, n);
        std::process::exit(1);
    }
}

// COLOR

fn color_start(color: Color) -> String {
    let addition = if color < BlackBg {
        30
    } else if color < BlackBright {
        40 - (BlackBg as i32)
    } else if color < BlackBrightBg {
        90 - (BlackBright as i32)
    } else {
        100 - (BlackBrightBg as i32)
    };
    let code = addition + color as i32;
    return format!("\x1b[{}m", code);
}

const color_end: &str = "\x1b[0m";

#[allow(dead_code)]
#[derive(PartialEq, PartialOrd)]
enum Color {
    Black, Red, Green, Yellow, Blue, Magenta, Cyan, White,
    BlackBg, RedBg, GreenBg, YellowBg, BlueBg, MagentaBg, CyanBg, WhiteBg,
    BlackBright, RedBright, GreenBright, YellowBright, BlueBright, MagentaBright, CyanBright, WhiteBright,
    BlackBrightBg, RedBrightBg, GreenBrightBg, YellowBrightBg, BlueBrightBg, MagentaBrightBg, CyanBrightBg, WhiteBrightBg,
}

// REQUEST

fn make_http_request() -> Result<String, parser::Error> {
    Ok(minreq::get("https://whoniverse-app.com/calcal/main.php")
        .send()
        .map_err(|_e| { parser::Error::InvalidResponse })?
        .as_str()
        .map_err(|_e| { parser::Error::ExpectedEOF })?
        .to_owned())
}

// todo: first entry is missing

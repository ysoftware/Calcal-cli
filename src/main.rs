mod parser;
mod terminal;

use Color::*;

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

    enter_draw_loop(entries);
    terminal::restore_terminal();
}

fn enter_draw_loop(entries: Vec<parser::EntryEntity>) {
    let mut input: char = ' ';
    let mut input_buffer = "".to_string();
    
    let mut needs_redraw = true;
    let (mut width, mut height) = terminal::get_window_size();
    let mut selected_entry_index = entries.len() - 1;

    loop {
        if needs_redraw {
            terminal::clear_window();

            if height > 40 && width > 70 {
                draw_empty();
            }

            let selected_entry = &entries[selected_entry_index];

            let mut entry_calories = 0.0;

            for section in &selected_entry.sections {
                for item in &section.items {
                    entry_calories += item.calories;
                }
            }

            draw_line_right(
                format!("{}", selected_entry.date), BlackBg,
                format!("{} kcal", entry_calories), BlueBg,
                width, 0
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
                    width, 
                    20 // align right text in the middle of the line
                );

                for i in 0..section.items.len() {
                    let item = &section.items[i];
                    let left_color = if i % 2 == 1 { White } else { BlackBg };
                    let right_color = if i % 2 == 1 { White } else { BlackBg };

                    draw_line_right(
                        format!("- {}, {} {}", item.title, item.quantity, item.measurement), left_color,
                        format!("{} kcal", item.calories), right_color,
                        width, 0
                    );
                }
            }
        }

        // handle input
        let has_new_input: bool;
        if let Some(new_char) = terminal::get_input() {
            has_new_input = true;
            input = new_char;
            input_buffer.push(input);

            if input == '\n' {
                input_buffer = "".to_string();
            }

            if input as usize == 68 { // arrow left
                if selected_entry_index - 1 > 0 {
                    selected_entry_index -= 1;
                }
            }

            if input as usize == 67 { // arrow right
                if selected_entry_index + 1 < entries.len() {
                    selected_entry_index += 1;
                }
            }
        } else {
            has_new_input = false;
        }

        let (new_width, new_height) = terminal::get_window_size();
        let did_resize_window = new_width != width || new_height != height;
        width = new_width;
        height = new_height;

        needs_redraw = did_resize_window || has_new_input;

        if !needs_redraw {
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
    }
}

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


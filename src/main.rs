mod parser;
mod terminal;
use terminal::{ Color, Color::*, color_start, COLOR_END, as_char };

#[allow(unused_imports)]
use std::process::exit;

const DRAW_WIDTH: usize = 52;

enum State { List, Input, Calendar }

struct App {
    should_exit: bool,
    state: State,
    entries: Vec<parser::EntryEntity>,
    width: usize,
    height: usize,
    list: List,
    input: Input,
    calendar: Vec<CalendarMonth>,
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

    let input = initial_input_value(&entries);
    let list = initial_list_value(&entries);

    let mut app = App {
        should_exit: false,
        state: State::List,
        entries: entries,
        width: 0,
        height: 0,
        list: list,
        input: input,
        calendar: vec![],
    };

    loop {
        if process_input(&mut app) {
            if app.should_exit { break }
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

    if let Some(input) = terminal::get_input() {
        let did_process_input: bool = match app.state {
            State::List => { process_input_list(app, input) },
            State::Input => { process_input_input(app, input) },
            State::Calendar => { process_input_calendar(app, input) },
        };
        return did_resize_window || did_process_input;
    }
    
    return did_resize_window;
}

fn draw(app: &App) {
    terminal::clear_window();

    match app.state {
        State::List => { draw_list(&app); },
        State::Input => { draw_input(&app); },
        State::Calendar => { draw_calendar(&app); },
    }
}

// LIST VIEW

struct List {
    selected_entry_index: usize,
    item_deletion_index: i32,
    is_showing_deletion_alert: bool,
}

fn initial_list_value(entries: &Vec<parser::EntryEntity>) -> List {
    // todo: will it crash if entires are empty?
    List {
        selected_entry_index: entries.len() - 1,
        item_deletion_index: -1,
        is_showing_deletion_alert: false
    }
}

fn process_input_list(app: &mut App, input: [u8; 4]) -> bool {
    let did_process_input: bool;
    did_process_input = true; // this should be inside of blocks to redraw only when needed
                              // but it is leading to slow scrolling through pages
    
    let char_input = as_char(input);
    let selected_entry_items_count = count_entry_items(&app.entries[app.list.selected_entry_index]);

    if app.list.is_showing_deletion_alert {
        if char_input == 'y' {
            // delete the item
        }
        app.list.is_showing_deletion_alert = false;
        app.list.item_deletion_index = -1;
        return true;
    }

    if input[0] == 10 { // Enter
        if app.list.item_deletion_index >= 0 {
            app.list.is_showing_deletion_alert = true;
        }
    } else if input[0] == 27 && input[1] == 91 {
        if input[2] == 68 { // arrow left
            if app.list.selected_entry_index > 0 {
                app.list.selected_entry_index -= 1;
                app.list.item_deletion_index = -1;
            }
        } else if input[2] == 67 { // arrow right
            if app.list.selected_entry_index + 1 < app.entries.len() {
                app.list.selected_entry_index += 1;
                app.list.item_deletion_index = -1;
            }
        } else if input[1] == 91 && input[2] == 66 { // arrow down
            if app.list.item_deletion_index > -1 {
                app.list.item_deletion_index -= 1;
            } else if selected_entry_items_count > 0 {
                app.list.item_deletion_index = (selected_entry_items_count - 1) as i32;
            }
        } else if input[1] == 91 && input[2] == 65 { // arrow up
            if selected_entry_items_count > 0 {
                if (app.list.item_deletion_index as usize) < selected_entry_items_count - 1 
                    || app.list.item_deletion_index == -1 {
                        app.list.item_deletion_index += 1;
                    } else {
                        app.list.item_deletion_index = -1;
                }
            }
        }
    } else if char_input == 'i' || char_input == 'ш' {
        app.state = State::Input;
        app.input.state = InputState::Name;
        app.input.completions = make_completions_for_item_name(&app.input.all_items);
        refresh_completions(app);
        app.list.item_deletion_index = -1;
        app.input.completion_index = 0;

        // todo: get last section id
        app.input.section_name = "Fake meal".to_string();
    } else if char_input == 's' || char_input == 'ы' { 
        app.state = State::Input;
        app.input.state = InputState::SectionName;
        app.input.completions = make_completions_for_section_name();
        refresh_completions(app);
        app.list.item_deletion_index = -1;
        app.input.completion_index = 0;
        app.input.section_name = "".to_string();
    } else if char_input == 'c' || char_input == 'с' { // latin and cyrillic c are different
        app.state = State::Calendar;
        app.calendar = process_calendar_data(&app.entries);
        app.list.item_deletion_index = -1;
    }
    else if char_input == 'q' || char_input == 'й' {
        app.should_exit = true;
    }
    return did_process_input;
}

fn draw_list(app: &App) {
    let mut drawn_lines = 0;

    if app.height > 30 && app.width > 50 { 
        drawn_lines += 1;
        draw_empty(); 
    }

    let selected_entry = &app.entries[app.list.selected_entry_index];
    let mut item_index = (count_entry_items(&selected_entry) as i32) - 1;
    let mut entry_calories = 0.0;

    for section in &selected_entry.sections {
        for item in &section.items {
            entry_calories += item.calories;
        }
    }

    drawn_lines += 1;
    draw_line_right(
        format!("{}", selected_entry.date), BlackBg,
        format!("{} kcal", entry_calories), BlueBrightBg,
        app.width, DRAW_WIDTH, 0
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
            app.width, DRAW_WIDTH,
            20 // align right text in the middle of the line
        );
        drawn_lines += 2;

        for i in 0..section.items.len() {
            let item = &section.items[i];
            let left_color = if i % 2 == 1 { White } else { BlackBg };

            if app.list.item_deletion_index == item_index as i32 {
                if app.list.is_showing_deletion_alert {
                    drawn_lines += 1;
                    draw_line_right(
                        format!("Delete {}?", item.title), RedBrightBg,
                        "[confirm: y]".to_string(), RedBrightBg,
                        app.width, DRAW_WIDTH, 0
                    );
                } else {
                    drawn_lines += 1;
                    draw_line_right(
                        format!("x {}, {} {}", item.title, item.quantity, item.measurement), left_color,
                        "[delete]".to_string(), RedBrightBg,
                        app.width, DRAW_WIDTH, 0
                    );
                }
            } else {
                let right_color = if i % 2 == 1 { White } else { BlackBg };
                drawn_lines += 1;
                draw_line_right(
                    format!("- {}, {} {}", item.title, item.quantity, item.measurement), left_color,
                    format!("{} kcal", item.calories), right_color,
                    app.width, DRAW_WIDTH, 0
                );
            }
            item_index -= 1;
        }
    }

    // todo: introduce vertical scrolling
    let used_lines = std::cmp::min(app.height, drawn_lines + 3); // cleanup: explain this magic number
    for _ in 0..app.height-used_lines { draw_empty(); }

    draw_empty();
    draw_line_right(
        "".to_string(), BlackBg, 
        format!("[{}/{}]", app.list.selected_entry_index + 1, app.entries.len()), BlackBrightBg,
        app.width, DRAW_WIDTH, 0
    );
}

fn count_entry_items(entry: &parser::EntryEntity) -> usize {
    let mut count = 0;
    for section in &entry.sections {
        for _ in &section.items {
            count += 1;
        }
    }
    return count;
}

// INPUT VIEW

#[allow(dead_code)]
enum InputState {
    SectionName, Name, Quantity, Calories
}

struct Input {
    state: InputState,
    query: String,
    completions: Vec<String>,
    filtered_completions: Vec<String>,
    all_items: Vec<parser::Item>,
    completion_index: i32,
    section_name: String,
}

fn initial_input_value(entries: &Vec<parser::EntryEntity>) -> Input {
    let mut all_items: Vec<parser::Item> = vec![];
    for entry in entries {
        for section in &entry.sections {
            for item in &section.items {
                all_items.push(item.clone());
            }
        }
    }

    return Input {
        state: InputState::Name,
        query: "".to_string(),
        completions: vec![],
        filtered_completions: vec![],
        all_items: all_items,
        completion_index: -1,
        section_name: "".to_string(),
    };
}

fn process_input_input(app: &mut App, input: [u8; 4]) -> bool {
    if input[0] == 10 { // Enter
        if app.input.completion_index > 0 {
            app.input.section_name = app.input.filtered_completions[app.input.completion_index as usize].clone();
            app.input.state = InputState::Name;
        } else {
            // save inputted value
        }
    } else if input[0] == 127 { // DEL // todo: not del?
        if app.input.query.len() > 0 {
            app.input.query.pop();
            app.input.completion_index = -1;
        }
    } else if !(input[0] > 0 && input[0] < 32) { // typing text
        app.input.query.push(as_char(input));
        app.input.completion_index = -1;
    } else if input[0] == 27 { // special characters
        if input[1] == 0 { // Esc
            app.state = State::List;
            app.input.query = "".to_string();
            app.input.completion_index = -1;
        } else if input[1] == 115 {
            app.input.query.push('ß');
            app.input.completion_index = -1;
        } else if input[1] == 101 {
            app.input.query.push('é');
            app.input.completion_index = -1;
        } else if input[1] == 117 {
            app.input.query.push('ü');
            app.input.completion_index = -1;
        } else if input[1] == 111 {
            app.input.query.push('ö'); // todo: type ó for Jamón
            app.input.completion_index = -1;
        } else if input[1] == 97 {
            app.input.query.push('ä');
            app.input.completion_index = -1;
        } else if input[1] == 91 && input[2] == 66 { // arrow down
            if app.input.completion_index > -1 {
                app.input.completion_index -= 1;
            } else if app.input.filtered_completions.len() > 0 {
                app.input.completion_index = (app.input.filtered_completions.len() - 1) as i32;
            }
        } else if input[1] == 91 && input[2] == 65 { // arrow up
            if app.input.filtered_completions.len() > 0 {
                if (app.input.completion_index as usize) < app.input.filtered_completions.len() - 1 
                 || app.input.completion_index == -1 {
                    app.input.completion_index += 1;
                } else {
                    app.input.completion_index = -1;
                }
            }
        } // todo: holding arrow up/down starts printing [[[[
    }

    refresh_completions(app);

    return true;
}

fn draw_input(app: &App) {
    // todo: introduce vertical scrolling
    let used_lines = std::cmp::min(app.height, app.input.filtered_completions.len() + 5); // cleanup: explain this magic number
    for _ in 0..app.height-used_lines { draw_empty(); }

    let completions_count = app.input.filtered_completions.len();
    for i in 0..completions_count {
        let reversed_index = completions_count-1-i;
        let completion = &app.input.filtered_completions[reversed_index];
        let color = if (reversed_index as i32) == app.input.completion_index { BlueBg } else { White };
        draw_line(format!("{}", completion), color, app.width, DRAW_WIDTH, 0);
    }

    draw_empty();
    draw_line_right(
        format!("> {}", app.input.query), BlackBg, 
        format!("[{}]", state_name(&app.input.state)), BlackBrightBg,
        app.width, DRAW_WIDTH, 0
    );

    draw_empty();
    if app.input.section_name.len() > 0 {
        draw_line(
            format!("Adding to {}", app.input.section_name), BlackBg, 
            app.width, DRAW_WIDTH, 0
        );
    } else {
        draw_line(
            "Adding new meal".to_string(), BlackBg, 
            app.width, DRAW_WIDTH, 0
        );
    }
}

fn refresh_completions(app: &mut App) {
    let clean_query = app.input.query.to_lowercase().to_string();
    if clean_query.len() > 0 {
        app.input.filtered_completions = vec![];
        for completion in &app.input.completions {
            if completion.to_lowercase().contains(&clean_query) {
                app.input.filtered_completions.push(completion.clone());
            }

            if app.input.filtered_completions.len() == 10 {
                break;
            }
        }
    } else {
        app.input.filtered_completions = app.input.completions.clone();

        if app.input.filtered_completions.len() > 10 {
            app.input.filtered_completions = app.input.completions[0..=10].to_vec();
        }
    }
}

fn state_name(state: &InputState) -> String {
    match &state {
        InputState::Name => "Item name",
        InputState::SectionName => "Meal name",
        InputState::Quantity => "Quantity",
        InputState::Calories => "Calories",
    }.to_string()
}

fn make_completions_for_item_name(all_items: &Vec<parser::Item>) -> Vec<String> {
    let mut result: Vec<String> = vec![];

    let mut seen = std::collections::HashSet::new();
    let mut unique_items = all_items.clone();
    unique_items.retain(|item| 
        seen.insert(format!("{} {}", item.title, item.measurement))
    );

    for item in unique_items { 
        result.push(format!("{} (in {})", item.title, item.measurement));
    }

    return result;
}

fn make_completions_for_section_name() -> Vec<String> {
    [ // cleanup: this is dumb how we have to to_string() everything here
        "Breakfast".to_string(), 
        "Lunch".to_string(), 
        "Dinner".to_string(), 
        "Snack".to_string(), 
        "Snack 2".to_string()
    ].to_vec()
}

// CALENDAR VIEW

struct CalendarMonth {
    title: String,
    average: f32,
    rows: Vec<[CalendarCell; 7]>
}

#[derive(Clone)]
struct CalendarCell {
    color: Color,
    text: String
}

fn process_input_calendar(app: &mut App, input: [u8; 4]) -> bool {
    if input[0] == 27 && input[1] == 0 { // ESC
        app.state = State::List;
    }
    return true;
}

fn draw_calendar(app: &App) {
    if app.calendar.len() == 0 {
        println!("Calendar is empty");
        return;
    }

    let cell_target_width = if app.width > 48 { 6 }
    else if app.width > 33 { 4 }
    else { 2 };

    let spacing = if app.width > 33 { 1 } else { 0 };

    let max_draw_width = std::cmp::min(app.width, 50);
    let cell_width = std::cmp::min(cell_target_width, (max_draw_width - spacing * 6) / 7);
    let draw_width = (cell_width * 7) + (spacing * 6);

    let empty_width = app.width - draw_width;
    let left_side_spacing = if empty_width > 1 { empty_width / 2 } else { 0 };

    draw_empty();

    for month in &app.calendar {
        let subtitle: String = if app.width > 33 {
            format!("∅ {:.0}", month.average)
        } else {
            format!("∅{:.0}", month.average)
        };

        draw_line_right(
            month.title.to_string(), White,
            subtitle, White,
            app.width, draw_width, 0
        );

        if app.width > 33 { draw_empty(); }
        
        for i in 0..month.rows.len() {
            let row = &month.rows[i];

            for _ in 0..left_side_spacing { print!(" "); }
            for j in 0..7 {
                if cell_width < 4 {
                    print!("{}", color_start(&row[j].color)); 
                        for _ in 0..cell_width { print!(" ") };
                     print!("{}", COLOR_END);
                } else {
                    let text_length = row[j].text.len();
                    let padding_start = (cell_width - text_length) / 2;
                    let padding_end = cell_width - text_length - padding_start;

                    print!("{}", color_start(&row[j].color));
                    for _ in 0..padding_start { print!(" ") };
                    print!("{}", &row[j].text);
                    for _ in 0..padding_end { print!(" ") };
                    print!("{}", COLOR_END);
                    if j < 6 { for _ in 0..spacing { print!(" "); } }
                }
            }
            println!("");

            if cell_width >= 4 {
                draw_empty();
            }
        }
        draw_empty();
    }
}

fn process_calendar_data(entries: &Vec<parser::EntryEntity>) -> Vec<CalendarMonth> {
    let mut months = vec![];

    if entries.len() == 0 {
        return months;
    }

    let mut current_month = get_month_component(&entries[0].date);
    let mut rows: Vec<[CalendarCell; 7]> = vec![];

    let mut i = 0;
    while i < entries.len() {
        let mut columns: [CalendarCell; 7] = [ // cleanup: this is dumb, but I can't clone a String?
            CalendarCell { color: White, text: "    ".to_string() },
            CalendarCell { color: White, text: "    ".to_string() },
            CalendarCell { color: White, text: "    ".to_string() },
            CalendarCell { color: White, text: "    ".to_string() },
            CalendarCell { color: White, text: "    ".to_string() },
            CalendarCell { color: White, text: "    ".to_string() },
            CalendarCell { color: White, text: "    ".to_string() },
        ];

        let mut columns_added = false;
        for w in 0..=6 {
            if entries.len() <= i {
                i += 1;
                continue;
            }

            let month = get_month_component(&entries[i].date);

            if current_month != month {
                if rows.len() > 0 || columns_added {
                    if columns_added {
                        rows.push(columns);

                        columns = [ // cleanup: this is dumb, but I can't clone a String?
                            CalendarCell { color: White, text: "    ".to_string() },
                            CalendarCell { color: White, text: "    ".to_string() },
                            CalendarCell { color: White, text: "    ".to_string() },
                            CalendarCell { color: White, text: "    ".to_string() },
                            CalendarCell { color: White, text: "    ".to_string() },
                            CalendarCell { color: White, text: "    ".to_string() },
                            CalendarCell { color: White, text: "    ".to_string() },
                        ];
                    }

                    let mut total = 0.0;
                    let mut count = 0;
                    for row in &rows {
                        for cell in row {
                            if cell.text.trim().len() > 0 {
                                total += cell.text.trim().parse::<f32>().unwrap();
                                count += 1;
                            }
                        }
                    }
                    let average_calories = total / count as f32;

                    months.push(CalendarMonth {
                        title: month_from_number(current_month).to_string(),
                        average: average_calories,
                        rows: rows
                    });
                    rows = vec![];
                    columns_added = false;
                }
                current_month = month;
                continue;
            }

            let mut calories = 0.0;
            for section in &entries[i].sections {
                for item in &section.items {
                    calories += item.calories;
                }
            }

            let weekday = get_weekday_component(&entries[i].date);
            
            if weekday == w {
                columns_added = true;
                columns[w as usize] = CalendarCell {
                    color: color_for_calories(calories),
                    text: format!("{}", calories)
                };
                i += 1;
                continue;
            } 
        }

        if columns_added {
            rows.push(columns);
        }
    }

    if rows.len() > 0 {
        let mut total = 0.0;
        let mut count = 0;
        for row in &rows {
            for cell in row {
                if cell.text.trim().len() > 0 {
                    total += cell.text.trim().parse::<f32>().unwrap();
                    count += 1;
                }
            }
        }
        let average_calories = total / count as f32;

        months.push(
            CalendarMonth {
                title: month_from_number(current_month).to_string(),
                average: average_calories,
                rows: rows
            }
        );
    }
    
    // std::process::exit(1);
    return months;
}

fn get_month_component(date: &str) -> i32 {
    use chrono::{NaiveDate, Datelike};
    let parsed_date = NaiveDate::parse_from_str(date, "%d %B %Y").unwrap();
    return parsed_date.month() as i32;
}

fn get_weekday_component(date: &str) -> i32 { // 0 - 6
    use chrono::{NaiveDate, Datelike};
    let parsed_date = NaiveDate::parse_from_str(date, "%d %B %Y").unwrap();
    return parsed_date.weekday() as i32;
}

fn month_from_number(month: i32) -> &'static str {
   match month {
       1 => { return "January" },
       2 => { return "February" },
       3 => { return "March" },
       4 => { return "April" },
       5 => { return "May" },
       6 => { return "June" },
       7 => { return "July" },
       8 => { return "August" },
       9 => { return "September" },
       10 => { return "October" },
       11 => { return "November" },
       12 => { return "December" },

       i32::MIN..=0_i32 => { return "" },
       13..=i32::MAX => { return "" },
   }
}

fn color_for_calories(calories: f32) -> Color {
    if calories <= 1400.0 {
        return BlackBg;
    } else if calories <= 1900.0 {
        return GreenBrightBg;
    } else if calories <= 2200.0 {
        return GreenBg;
    } else if calories <= 2400.0 {
        return YellowBrightBg;
    } else if calories <= 2900.0 {
        return RedBrightBg;
    } else {
        return RedBg;
    }
}

// DRAW TEXT

fn draw_empty() {
    println!("");
}

fn draw_line(
    string: String,
    color: Color,
    window_width: usize,
    max_draw_width: usize,
    left_limit: usize
) {

    draw_line_right(
        string, color, "".to_string(), White,
        window_width, max_draw_width, left_limit
    );
}

fn draw_line_right(
    string_left: String,
    color_left: Color,
    string_right: String,
    color_right: Color,
    window_width: usize,
    max_draw_width: usize,
    left_limit: usize
) {
    let draw_width = std::cmp::min(max_draw_width, window_width);
    let empty_width = window_width - draw_width;

    let left_side_padding = if empty_width > 1 { empty_width / 2 } else { 0 };
    for _ in 0..left_side_padding { print!(" "); }

    let length_left = string_left.chars().count();
    let length_right = string_right.chars().count();
    let padding = ".. ";

    if length_left + length_right + padding.len() <= draw_width {
        print!("{}{}", color_start(&color_left), string_left);

        let mut spacing = draw_width - (length_left + length_right);
        if left_limit > 0 && spacing > 1 { // todo: spacing > 1? >2?
            spacing = std::cmp::min(spacing, left_limit - (length_left));
        }
        for _ in 0..spacing { print!(" "); }

        println!("{}{}{}", color_start(&color_right), string_right, COLOR_END);
    } else {
        if length_right <= draw_width {
            if length_right + padding.len() < draw_width {
                let rest_width = draw_width - length_right - padding.len();
                let truncated_string = truncate(string_left, rest_width);
                print!("{}{}{}{}", color_start(&color_left), truncated_string, padding, COLOR_END);
            }

            println!("{}{}{}", color_start(&color_right), string_right, COLOR_END);
        } else {
            let truncated_string = truncate(string_right, draw_width);
            println!("{}{}{}", color_start(&color_right), truncated_string, COLOR_END);
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

// REQUEST

fn make_http_request() -> Result<String, parser::Error> {
    Ok(minreq::get("https://whoniverse-app.com/calcal/main.php")
        .send()
        .map_err(|_e| { parser::Error::InvalidResponse })?
        .as_str()
        .map_err(|_e| { parser::Error::ExpectedEOF })?
        .to_owned())
}

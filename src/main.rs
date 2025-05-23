mod parser;
mod terminal;
use terminal::{ Color, Color::*, color_start, COLOR_END, as_char };
use parser::{QuantityMeasurement, Item, measurement_display_value};
use std::collections::HashMap;

// TODO: input calories with / to recalculate per weight

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
    calendar: Calendar,
}

fn main() {
    parser::test_all();

    #[allow(invalid_value)] // zero init, don't tell me what to do
    let mut app: App = unsafe { std::mem::MaybeUninit::zeroed().assume_init() };

    download_data(&mut app);

    terminal::prepare_terminal();
    terminal::clear_window();

    loop {
        let mut should_draw = process_window_resize(&mut app);

        if let Some(input) = terminal::get_input() {
            let did_process_input: bool = match app.state {
                State::List => { process_input_list(&mut app, input) },
                State::Input => { process_input_input(&mut app, input) },
                State::Calendar => { process_input_calendar(&mut app, input) },
            };
            // TODO: fix not refreshing on irrelevant input
            if did_process_input {
                should_draw = true;
            }
        }

        if should_draw {
            if app.should_exit { break }

            terminal::clear_window();
            match app.state {
                State::List => { draw_list(&app); },
                State::Input => { draw_input(&app); },
                State::Calendar => { draw_calendar(&app); },
            }
        } else {
            std::thread::sleep(std::time::Duration::from_millis(50));
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

// LIST VIEW

struct List {
    selected_entry_index: usize,
    item_deletion_index: i32,
    is_showing_deletion_alert: bool,
}

fn process_input_list(app: &mut App, input: [u8; 4]) -> bool {
    let did_process_input: bool;
    did_process_input = true; // this should be inside of blocks to redraw only when needed
                              // but it is leading to slow scrolling through pages
    
    let char_input = as_char(input);
    let selected_entry_items_count = count_entry_items(&app.entries[app.list.selected_entry_index]);

    if app.list.is_showing_deletion_alert {
        if char_input == 'y' {
            let entry = &app.entries[app.list.selected_entry_index];
            let mut count = 0;
            let mut section_index = entry.sections.len() - 1;
            let mut item_index: i64 = 0;

            // find selected section_index and item_index from item_deletion_index (counts backwards)
            'outer: for s_idx in (0..entry.sections.len()).rev() {
                let items_count = entry.sections[s_idx].items.len();
                item_index = (items_count as i64) - 1;
                for _ in (0..items_count).rev() {
                    if count == app.list.item_deletion_index {
                        break 'outer;
                    }
                    count += 1;
                    item_index -= 1;
                }
                item_index = 0;
                section_index -= 1;
            }

            if entry.sections[section_index].items.len() == 1 {
                app.entries[app.list.selected_entry_index].sections.remove(section_index);
            } else {
                app.entries[app.list.selected_entry_index].sections[section_index].items.remove(item_index as usize);
            }
            upload_data(app.entries[app.list.selected_entry_index].clone());
        }
        app.list.is_showing_deletion_alert = false;
        app.list.item_deletion_index = -1;
        return true;
    }

    if char_input == 'd' {
        if app.list.item_deletion_index >= 0 {
            app.list.is_showing_deletion_alert = true;
        }
    } else if input[0] == 27 && input[1] == 0 { // Esc
        if app.list.item_deletion_index >= 0 {
            app.list.item_deletion_index = -1;
        }
    } else if char_input == 'h' || (input[1] == 91 && input[2] == 68) { // h or arrow left
        if app.list.selected_entry_index > 0 {
            app.list.selected_entry_index -= 1;
            app.list.item_deletion_index = -1;
        }
    } else if char_input == 'l' || (input[1] == 91 && input[2] == 67) { // l or arrow right
        if app.list.selected_entry_index + 1 < app.entries.len() {
            app.list.selected_entry_index += 1;
            app.list.item_deletion_index = -1;
        }
    } else if char_input == 'j' || (input[1] == 91 && input[2] == 66) { // j or arrow down
        if app.list.item_deletion_index > -1 {
            app.list.item_deletion_index -= 1;
        } else if selected_entry_items_count > 0 {
            app.list.item_deletion_index = (selected_entry_items_count - 1) as i32;
        }
    } else if char_input == 'k' || (input[1] == 91 && input[2] == 65) { // k or arrow up
        if selected_entry_items_count > 0 {
            if (app.list.item_deletion_index as usize) < selected_entry_items_count - 1 
                || app.list.item_deletion_index == -1 {
                    app.list.item_deletion_index += 1;
                } else {
                    app.list.item_deletion_index = -1;
            }
        }
    } else if char_input == 'i' || char_input == 'ш' {
        app.state = State::Input;
        app.input.state = InputState::Name;
        app.input.completions = make_completions_for_item_name(&app.input.all_items);
        refresh_completions(app);
        app.list.item_deletion_index = -1;
        app.input.completion_index = 0;

        if let Some(section) = app.entries[app.list.selected_entry_index].sections.last() {
            app.input.section_name = section.id.clone();
        } else {
            app.input.section_name = "Breakfast".to_string();
        }
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
        app.calendar.scroll_offset = 0;
        app.calendar.months = process_calendar_data(&app.entries);
        app.list.item_deletion_index = -1;
    } else if char_input == 'r' {
        download_data(app);
    } else if char_input == 'n' || char_input == 'т' {
        let today_string = get_today_string();
        if app.entries.last().unwrap().date != today_string {
            let entry = parser::EntryEntity {
                date: today_string,
                sections: vec![]
            };
            app.entries.push(entry.clone());
            app.list.selected_entry_index = app.entries.len() - 1;
            upload_data(entry);
        }
    } else if char_input == 'q' || char_input == 'й' {
        app.should_exit = true;
    }

    return did_process_input;
}

// TODO: round calories (also in the exported data)
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

        // TODO: draw measurement in its own column
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
                        format!("> {}, {}", 
                            item.title, measurement_display_value(&item.quantity, &item.measurement)
                        ), left_color,
                        "[delete]".to_string(), RedBrightBg,
                        app.width, DRAW_WIDTH, 0
                    );
                }
            } else {
                let right_color = if i % 2 == 1 { White } else { BlackBg };
                drawn_lines += 1;
                draw_line_right(
                    format!("- {}, {}", 
                        item.title, measurement_display_value(&item.quantity, &item.measurement)
                    ), left_color,
                    format!("{} kcal", item.calories), right_color,
                    app.width, DRAW_WIDTH, 0
                );
            }
            item_index -= 1;
        }
    }

    // TODO: introduce vertical scrolling
    let used_lines = std::cmp::min(app.height, drawn_lines + 3); // cleanup: explain this magic number
    for _ in 0..app.height-used_lines { draw_empty(); }
    
    let today_string = get_today_string();
    let should_show_add_day = app.entries.last().unwrap().date != today_string;
    let status_line_left = if should_show_add_day { 
        format!("Press n to add {}", today_string)
    } else if app.list.item_deletion_index >= 0 {
        if app.list.is_showing_deletion_alert {
            "Press y to confirm deletion".to_string()
        } else {
            "Press d to delete".to_string()
        }
    } else { 
        "".to_string() 
    };

    draw_empty();

    draw_line_right(
        status_line_left, BlackBg, 
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

#[derive(PartialEq)]
enum InputState {
    SectionName, Name, Quantity, Calories
}

#[derive(Clone)]
struct Completion {
    label: String,
    filter: String,
    item: Option<Item>,
}

struct Input {
    state: InputState,
    query: String,
    completions: Vec<Completion>,
    filtered_completions: Vec<Completion>,
    all_items: Vec<Item>,
    completion_index: i32,
    section_name: String,
    name: String,
    calories: f32,
    quantity: f32,
    measurement: QuantityMeasurement,
}

fn process_input_input(app: &mut App, input: [u8; 4]) -> bool {
    if input[0] == 10 { // Enter
        match app.input.state {
            InputState::SectionName => {
                if app.input.completion_index >= 0 {
                        app.input.section_name = app.input.filtered_completions[app.input.completion_index as usize].label.clone();
                } else if app.input.query.len() > 0 {
                    // TODO: trim and capitalise?
                    app.input.section_name = app.input.query.clone();
                } else {
                    return true; // discard input
                }
                app.input.state = InputState::Name;
                app.input.completion_index = -1;
                app.input.completions = make_completions_for_item_name(&app.input.all_items);
                app.input.query = "".to_string();
                refresh_completions(app);
            },
            InputState::Name => {
                if app.input.query.len() > 0 {
                    if app.input.completion_index >= 0 {
                        app.input.name = app.input.filtered_completions[app.input.completion_index as usize].item.as_ref().unwrap().title.clone();
                    } else {
                        // TODO: trim and capitalise?
                        app.input.name = app.input.query.clone();
                    }
                } else if app.input.completion_index >= 0 {
                    let section_name = app.input.section_name.clone();
                    append_item(
                        app,
                        app.list.selected_entry_index,
                        &section_name,
                        app.input.filtered_completions[app.input.completion_index as usize].item.as_ref().unwrap().clone()
                    );
                    upload_data(app.entries.last().unwrap().clone());
                    return true;
                } else {
                    return true; // discard input
                }
                app.input.state = InputState::Quantity;
                app.input.completion_index = -1;
                app.input.completions = make_completions_for_quantity(&app.input.all_items, &app.input.name);
                app.input.query = "".to_string();
                refresh_completions(app);
            },
            InputState::Quantity => {
                if app.input.completion_index >= 0 {
                    let item = app.input.completions[app.input.completion_index as usize].item.as_ref().unwrap();
                    app.input.quantity = item.quantity;
                    app.input.measurement = item.measurement.clone();
                } else {
                    if let Some(value) = parser::get_quantity(&app.input.query) {
                        app.input.quantity = value.0;
                        app.input.measurement = value.1;
                    }
                }

                // TODO: round calories the same way as in Swift
                // try to calculate calories for the user
                for item in &app.input.all_items {
                    if item.title.to_lowercase() == app.input.name.to_lowercase() && item.measurement == app.input.measurement {
                        append_item(
                            app,
                            app.list.selected_entry_index, 
                            &app.input.section_name.clone(),
                            Item {
                                title: app.input.name.clone(),
                                calories: (item.calories / item.quantity * app.input.quantity).ceil(),
                                measurement: app.input.measurement.clone(),
                                quantity: app.input.quantity.clone(),
                            }
                        );
                        upload_data(app.entries.last().unwrap().clone());
                        return true;
                    }
                }

                app.input.state = InputState::Calories;
                app.input.completion_index = -1;
                app.input.completions = vec![];
                app.input.query = "".to_string();
                refresh_completions(app);
            },
            InputState::Calories => {
                if app.input.completion_index >= 0 {
                    let item = app.input.completions[app.input.completion_index as usize].item.as_ref().unwrap();
                    app.input.calories = item.calories;
                } else if let Ok(calories) = app.input.query.parse::<f32>() {
                    app.input.calories = calories;
                } else {
                    return true;
                }

                append_item(
                    app,
                    app.list.selected_entry_index, 
                    &app.input.section_name.clone(),
                    Item {
                        title: app.input.name.clone(),
                        calories: app.input.calories.clone(),
                        measurement: app.input.measurement.clone(),
                        quantity: app.input.quantity.clone(),
                    }
                );

                upload_data(app.entries.last().unwrap().clone());
                return true;
            },
        }
    } else if input[0] == 127 && (input[1] == 0 || input[1] == 127) && input[2] == 0 && input[3] == 0 { // backspace
        if app.input.query.len() > 0 {
            app.input.query.pop();
            app.input.completion_index = -1;
        }
    } else if (input[0] >= 208 && input[0] <= 209 && input[1] >= 128 && input[1] <= 200) // cyrillic alphabet
                || (input[0] >= 32 && input[0] < 127) { // latin alphabet
        app.input.query.push(as_char(input));
        app.input.completion_index = -1;
    } else if input[0] == 15 && input[1] == 0 && input[2] == 0 && input[3] == 0 {
        app.input.query.push('ó'); // ctrl+o
        app.input.completion_index = -1;
    } else if input[0] == 5 && input[1] == 0 && input[2] == 0 && input[3] == 0 {
        app.input.query.push('é'); // ctrl+e
        app.input.completion_index = -1;
    } else if input[0] == 27 { // special characters
        if input[1] == 0 { // Esc
            app.state = State::List;
            app.input.query = "".to_string();
            app.input.completion_index = -1;
        } else if input[1] == 115 {
            app.input.query.push('ß'); // alt+s
            app.input.completion_index = -1;
        } else if input[1] == 117 {
            app.input.query.push('ü'); // alt+u
            app.input.completion_index = -1;
        } else if input[1] == 111 {
            app.input.query.push('ö'); // alt+o
            app.input.completion_index = -1;
        } else if input[1] == 97 {
            app.input.query.push('ä'); // alt+a
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
        } // TODO: holding arrow up/down starts printing [[[[
    }

    refresh_completions(app);

    return true;
}

fn draw_input(app: &App) {
    let used_lines = std::cmp::min(app.height, app.input.filtered_completions.len() + 5); // cleanup: explain this magic number
    for _ in 0..app.height-used_lines { draw_empty(); }

    let completions_count = app.input.filtered_completions.len();
    for i in 0..completions_count {
        let reversed_index = completions_count-1-i;
        let completion = &app.input.filtered_completions[reversed_index];
        let color = if (reversed_index as i32) == app.input.completion_index { BlueBg } else { White };

        match &app.input.state {
            InputState::SectionName => {
                if app.input.query.len() > 0 {
                    draw_line(
                        format!("{}", completion.label), color, 
                        app.width, DRAW_WIDTH, 0
                    );
                } else {
                    draw_line(
                        completion.label.clone(), color, 
                        app.width, DRAW_WIDTH, 0
                    );
                }
            },
            InputState::Name => {
                if app.input.query.len() > 0 {
                    draw_line(
                        format!("{}", completion.item.as_ref().unwrap().title), color, 
                        app.width, DRAW_WIDTH, 0
                    );
                } else {
                    draw_line(
                        completion.label.clone(), color, 
                        app.width, DRAW_WIDTH, 0
                    );
                }
            },
            InputState::Calories => {
                draw_line(
                    format!("{}", completion.item.as_ref().unwrap().title), color, 
                    app.width, DRAW_WIDTH, 0
                );
            },
            InputState::Quantity => {
                let item = completion.item.as_ref().unwrap();
                draw_line(
                    format!("{}, {} -> {} kcal", 
                        item.title, measurement_display_value(&item.quantity, &item.measurement), item.calories
                    ), color, 
                    app.width, DRAW_WIDTH, 0
                );
            },
        }
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
            format!("Adding to {} on {}", app.input.section_name, app.entries[app.list.selected_entry_index].date), BlackBg,
            app.width, DRAW_WIDTH, 0
        );
    } else {
        draw_line(
            format!("Adding new meal on {}", app.entries[app.list.selected_entry_index].date), BlackBg,
            app.width, DRAW_WIDTH, 0
        );
    }
}

fn append_item(app: &mut App, entry_index: usize, section_id: &String, item: Item) {
    let entry = &app.entries[entry_index];
    if let Some(section_index) = &entry.sections.iter().position(|section| section.id == *section_id) {
        app.entries[entry_index].sections[*section_index].items.push(item.clone());
        app.state = State::List;
        app.input.query = "".to_string();
        app.input.completion_index = -1;
    } else {
        let items = [item.clone()].to_vec();
        let new_section = parser::EntrySectionEntity { 
            id: section_id.to_string(), 
            items 
        };
        app.entries[entry_index].sections.push(new_section);
        app.state = State::List;
        app.input.query = "".to_string();
        app.input.completion_index = -1;
    }
}

fn refresh_completions(app: &mut App) {
    let clean_query = app.input.query.to_lowercase().to_string();
    if clean_query.len() > 0 {
        app.input.filtered_completions = vec![];
        for completion in &app.input.completions {
            if completion.filter.to_lowercase().contains(&clean_query) {
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

fn make_completions_for_item_name(all_items: &Vec<Item>) -> Vec<Completion> {
    let mut dict: HashMap<String, (Completion, usize)> = HashMap::new();

    for item in all_items {
        let query = format!("{}", item.title);
        let completion = Completion {
            label: "".to_string(),
            filter: format!("{}", item.title),
            item: Some(item.clone()),
        };

        if !dict.contains_key(&query) {
            dict.insert(query, (completion, 1));
        } else {
            dict.get_mut(&query).unwrap().1 += 1;
        }
    }

    let mut values: Vec<(Completion, usize)> = dict.into_values().collect();
    values.sort_by(|lhs, rhs| 
        rhs.1.partial_cmp(&lhs.1).unwrap()
    );

    for i in 0..values.len() {
        if let Some(item) = &values[i].0.item {
            values[i].0.label = format!("{}, {}, {} kcal (x{})", 
                item.title, measurement_display_value(&item.quantity, &item.measurement), item.calories, values[i].1
            );
        }
    }

    return values.into_iter().map(|value| value.0).collect();
}

fn make_completions_for_section_name() -> Vec<Completion> {
    ["Breakfast", "Lunch", "Dinner", "Snack", "Snack 2"]
        .map(|label| 
            Completion {
                filter: label.to_string(),
                item: None,
                label: label.to_string()
            }
        ).to_vec()
}

fn make_completions_for_quantity(all_items: &Vec<Item>, item_name: &String) -> Vec<Completion> {
    let mut quantities: Vec<Completion> = vec![];

    'outer: for item in all_items {
        if item.title.to_lowercase() == item_name.to_lowercase() {
            let label = format!("{} {} {}", item.title.to_lowercase(), item.quantity, item.measurement);
            println!("{}", label);

            for quantity in &quantities {
                if quantity.label == label {
                    continue 'outer;
                }
            }

            let completion = Completion {
                label,
                filter: "".to_string(),
                item: Some(item.clone())
            };
            quantities.push(completion)
        }
    }

    quantities.sort_by(|lhs, rhs| 
        lhs.item.as_ref().unwrap().quantity.partial_cmp(&rhs.item.as_ref().unwrap().quantity).unwrap()
    );
    return quantities;
}

// CALENDAR VIEW

struct Calendar {
    scroll_offset: usize,
    months: Vec<CalendarMonth>,
}

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
    let char_input = as_char(input);

    if input[0] == 27 && input[1] == 0 { // ESC
        app.state = State::List;
    } else if char_input == 'j' || (input[1] == 91 && input[2] == 66) { // j or arrow down
        if app.calendar.scroll_offset > 0 {
            app.calendar.scroll_offset -= 1
        }
    } else if char_input == 'k' || (input[1] == 91 && input[2] == 65) { // k or arrow up
        if app.calendar.months.len() > app.calendar.scroll_offset + 1 {
            app.calendar.scroll_offset += 1
        }
    }
    return true;
}

fn draw_calendar(app: &App) {
    if app.calendar.months.len() == 0 {
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

    let last_month = if app.calendar.months.len() > app.calendar.scroll_offset {
        app.calendar.months.len() - app.calendar.scroll_offset
    } else { 
        1
    };

    for i in 0..last_month {
        let month = &app.calendar.months[i];

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
                            // TODO: skip missing days? I had this bug when I was missing september 2 and 3
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
                        rows
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
                rows
            }
        );
    }
    
    return months;
}

fn get_today_string() -> String {
    use chrono::{DateTime, offset::Utc};
    let system_time = std::time::SystemTime::now();
    let datetime: DateTime<Utc> = system_time.into();
    return datetime.format("%-d %B %Y").to_string();
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
        return GreenBg;
    } else if calories <= 2200.0 {
        return CyanBg;
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
        if left_limit > 0 && spacing > 1 { // TODO: spacing > 1? >2?
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
        terminal::restore_terminal();
        exit(1);
    }
}

// REQUEST

const URL: &str = "http://ysoftware.online/main.php";
// const URL: &str = "http://localhost:7777/main.php";

fn get_data() -> Result<String, parser::Error> {
    Ok(minreq::get(URL)
        .send()
        .map_err(|_e| { parser::Error::InvalidResponse })?
        .as_str()
        .map_err(|_e| { parser::Error::ExpectedEOF })?
        .to_owned())
}

fn post_data(content: String) -> Result<minreq::Response, minreq::Error> {
    let password = std::fs::read_to_string("password.txt").expect("Missing password.txt file.");
    let boundary = "REQUEST_BOUNDARY";

    let mut body = "".to_string();

    // content
    body += &format!("--{}\r\n", boundary);
    body += "Content-Disposition: form-data; name=\"file\"; filename=\"text.txt\"\r\n";
    body += "Content-Type: text/plain\r\n\r\n";
    body += &content.trim();
    body += "\r\n";

    // password
    body += &format!("--{}\r\n", boundary);
    body += "Content-Disposition: form-data; name=\"password\"\r\n";
    body += "Content-Type: text/plain\r\n\r\n";
    body += &password.trim();
    body += "\r\n";

    body += &format!("--{}--\r\n", boundary);
    
    return minreq::post(URL)
        .with_header("Content-Type", format!("multipart/form-data; boundary={}", boundary))
        .with_body(body)
        .send();
}

fn download_data(app: &mut App) {
    terminal::clear_window();
    println!("Loading data...");

    let response_string = get_data().unwrap_or_else(|error| {
        eprintln!("An error occured while making http request: {error}");
        terminal::restore_terminal();
        exit(1);
    });

    let entries = parser::parse_entities(response_string).unwrap_or_else(|error| {
        eprintln!("An error occured while parsing response: {error}");
        terminal::restore_terminal();
        exit(1);
    });

    for entry in &entries {
        for section in &entry.sections {
            for item in &section.items {
                app.input.all_items.push(item.clone());
            }
        }
    }

    app.input.query = "".to_string();
    app.list.selected_entry_index = entries.len() - 1;
    app.list.item_deletion_index = -1;
    app.entries = entries;
    app.state = State::List;
}

fn upload_data(entry: parser::EntryEntity) {
    terminal::clear_window();
    println!("Uploading data...");

    let entries = vec![entry];
    let data = parser::encode_entries(&entries);
    let response = post_data(data).unwrap_or_else(|error| {
        eprintln!("An error occured while making http request: {error}");
        terminal::restore_terminal();
        exit(1);
    });

    if response.status_code != 200 {
        for (key, value) in response.headers {
            eprintln!("{}: {}", key, value);
        }
        eprintln!("{}", response.reason_phrase);
        eprintln!("Error code {} - {} when posting data to: {}", response.status_code, response.reason_phrase, URL);
        terminal::restore_terminal();
        exit(1);
    }
}

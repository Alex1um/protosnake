use ncurses::*;
use crate::tui::err::print_error;

pub struct NumInput<'a> {
    pub name: &'a str,
    pub value: i32,
    pub raw: String,
    is_number: bool,
}

impl NumInput<'_> {
    pub fn new<'a>(name: &'a str) -> NumInput<'a> {
        NumInput { name, value: i32::default(), raw: String::new(), is_number: true }
    }
    
    pub fn default<'a>(name: &'a str, value: i32) -> NumInput<'a> {
        NumInput { name, value: value, raw: value.to_string(), is_number: true }
    }
    
    pub fn str<'a>(name: &'a str) -> NumInput<'a> {
        NumInput { name, value: i32::default(), raw: String::new(), is_number: false }
    }
    
    pub fn str_default<'a>(name: &'a str, value: &'a str) -> NumInput<'a> {
        NumInput { name, value: i32::default(), raw: String::from(value), is_number: false }
    }

}

pub fn show_menu_config(inputs: &mut Vec<NumInput>) -> Result<(), ()> {
    let mut selected = 0;
    let len = inputs.len() + 1;

    const INPUT_PAIR: i16 = 1;
    const INPUT_SELECTED_PAIR: i16 = 2;
    const SUMBMIT_PAIR: i16 = 3;
    const CANCEL_PAIR: i16 = 4;
    start_color();
    init_pair(INPUT_PAIR, COLOR_WHITE, COLOR_BLACK | 0b1000);
    init_pair(INPUT_SELECTED_PAIR, COLOR_WHITE, COLOR_BLUE);
    init_pair(SUMBMIT_PAIR, COLOR_BLACK, COLOR_GREEN | 0b1000);
    init_pair(CANCEL_PAIR, COLOR_BLACK, COLOR_RED | 0b1000);
    let buttons_row = inputs.len();
    let mut selected_button = 0;
    
    loop {
        clear();
        for (i, e) in inputs.iter().enumerate() {
            addstr(e.name);
            addstr("\n");
            if i == selected {
                attron(COLOR_PAIR(INPUT_SELECTED_PAIR));
                addstr(&format!("{:_<10}", e.raw));
                // e.print_nc();
                attroff(COLOR_PAIR(INPUT_SELECTED_PAIR));
            } else {
                attron(COLOR_PAIR(INPUT_PAIR));
                addstr(&format!("{:_<10}", e.raw));
                // e.print_nc();
                attroff(COLOR_PAIR(INPUT_PAIR));
            }
            addstr("\n");
        }
        if selected == buttons_row {
            if selected_button == 0 {
                attron(COLOR_PAIR(CANCEL_PAIR));
                addstr("Cancel");
                attroff(COLOR_PAIR(CANCEL_PAIR));
                addstr("  ");
                addstr("Submit");
            } else {
                addstr("Cancel");
                addstr("  ");
                attron(COLOR_PAIR(SUMBMIT_PAIR));
                addstr("Submit");
                attroff(COLOR_PAIR(SUMBMIT_PAIR));
            }
        } else {
            addstr("Cancel");
            addstr("  ");
            addstr("Submit");
        }
        
        refresh();

        let key = getch();
        match key {
            KEY_UP | 119 => {
                selected = selected + len - 1;
                selected %= len;
            }
            KEY_DOWN | 115 => {
                selected += 1;
                selected %= len;
            }
            KEY_LEFT | KEY_RIGHT => {
                selected_button ^= 1;
            }
            KEY_EXIT | KEY_CANCEL | KEY_CLOSE | KEY_EOS | KEY_BREAK => {
                return Err(())
            }
            48..=57 => { // digits
                let selected_raw = &mut inputs[selected].raw;
                selected_raw.push(char::from_u32(key as u32).unwrap());
            }
            97..=122 | 65..=90 | 95 | 32 => { 
                if !inputs[selected].is_number {
                    let selected_raw = &mut inputs[selected].raw;
                    selected_raw.push(char::from_u32(key as u32).unwrap());
                }
            }
            KEY_BACKSPACE | 264 => {
                let selected_raw = &mut inputs[selected].raw;
                selected_raw.pop();
            }
            KEY_ENTER | 10 => {
                if selected == buttons_row {
                    match selected_button {
                        0 => {
                            return Err(());
                            break;
                        }
                        1 => {
                            let mut all_valid = true;
                            for e in inputs.iter_mut() {
                                if e.is_number {
                                    match e.raw.parse() {
                                        Err(e) => {
                                            print_error(e);
                                            all_valid = false;
                                            break;
                                        }
                                        Ok(rs) => {
                                            e.value = rs;
                                        }
                                    } 
                                }
                            }
                            if all_valid {
                                break;
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => { }
        }
    }
    Ok(())
}
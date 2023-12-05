
use ncurses::*;
use crate::tui::err::print_error;

pub struct NumInput<'a> {
    pub name: &'a str,
    pub value: Option<i32>,
    raw: String,
}

impl NumInput<'_> {
    pub fn new<'a>(name: &'a str) -> NumInput<'a> {
        NumInput { name, value: None, raw: String::new() }
    }
    
    pub fn default<'a>(name: &'a str, value: i32) -> NumInput<'a> {
        NumInput { name, value: Some(value), raw: value.to_string() }
    }

    pub(self) fn print_nc(&self) {
        addstr(&self.raw);
    }
}

pub fn show_menu_config() -> Result<(), ()> {
    let mut input = NumInput::new("Direct connect");
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
    let BUTTONS_ROW = inputs.len();
    let mut selected_button = 0;
    
    loop {
        clear();
        addstr(input.name);
        addstr("\n");
        if 0 == selected {
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
        if selected == BUTTONS_ROW {
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
            }
            KEY_DOWN | 115 => {
                selected += 1;
            }
            KEY_LEFT | KEY_RIGHT => {
                selected_button ^= 1;
            }
            KEY_EXIT | KEY_CANCEL | KEY_CLOSE | KEY_EOS | KEY_BREAK => {
                return Err(())
            }
            48..=57 => { // digits
                if selected < inputs.len() {
                    let selected_raw = &mut inputs[selected].raw;
                    selected_raw.push(char::from_u32(key as u32).unwrap());
                }
            }
            KEY_BACKSPACE | 264 => {
                if selected < inputs.len() {
                    let selected_raw = &mut inputs[selected].raw;
                    selected_raw.pop();
                }
            }
            KEY_ENTER | 10 => {
                if selected == BUTTONS_ROW {
                    match selected_button {
                        0 => {
                            break;
                        }
                        1 => {
                            let mut all_valid = true;
                            for e in inputs.iter_mut() {
                                match e.raw.parse() {
                                    Err(e) => {
                                        print_error(e);
                                        all_valid = false;
                                        break;
                                    }
                                    Ok(rs) => {
                                        e.value = Some(rs);
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


        selected %= len;
    }


    Ok(input.value.unwrap())
}
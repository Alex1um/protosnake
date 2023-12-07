
use ncurses::*;
use crate::tui::err::print_error;

pub struct IpInput<'a> {
    pub name: &'a str,
    raw: String,
}

impl IpInput<'_> {
    pub fn new<'a>(name: &'a str) -> IpInput<'a> {
        IpInput { name, raw: String::new() }
    }
    
    pub fn default<'a>(name: &'a str, value: i32) -> IpInput<'a> {
        IpInput { name, raw: value.to_string() }
    }

    pub(self) fn print_nc(&self) {
        addstr(&self.raw);
    }
}

pub fn show_connect_dialog() -> Result<String, ()> {
    let mut input = IpInput::new("Direct connect");
    let mut input_selected = false;

    const INPUT_PAIR: i16 = 1;
    const INPUT_SELECTED_PAIR: i16 = 2;
    const SUMBMIT_PAIR: i16 = 3;
    const CANCEL_PAIR: i16 = 4;
    start_color();
    init_pair(INPUT_PAIR, COLOR_WHITE, COLOR_BLACK | 0b1000);
    init_pair(INPUT_SELECTED_PAIR, COLOR_WHITE, COLOR_BLUE);
    init_pair(SUMBMIT_PAIR, COLOR_BLACK, COLOR_GREEN | 0b1000);
    init_pair(CANCEL_PAIR, COLOR_BLACK, COLOR_RED | 0b1000);
    let mut selected_button = 0;
    
    loop {
        clear();
        addstr(input.name);
        addstr("\n");
        if input_selected {
            attron(COLOR_PAIR(INPUT_SELECTED_PAIR));
            addstr(&format!("{:_<20}", input.raw));
            // e.print_nc();
            attroff(COLOR_PAIR(INPUT_SELECTED_PAIR));
        } else {
            attron(COLOR_PAIR(INPUT_PAIR));
            addstr(&format!("{:_<20}", input.raw));
            // e.print_nc();
            attroff(COLOR_PAIR(INPUT_PAIR));
        }
        addstr("\n");
        if !input_selected {
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
            KEY_UP | 119 | KEY_DOWN | 115 => {
                input_selected = !input_selected;
            }
            KEY_LEFT | KEY_RIGHT => {
                selected_button ^= 1;
            }
            KEY_EXIT | KEY_CANCEL | KEY_CLOSE | KEY_EOS | KEY_BREAK => {
                return Err(())
            }
            48..=57 | 46 | 58 => { // digits | : | .
                input.raw.push(char::from_u32(key as u32).unwrap());
            }
            KEY_BACKSPACE | 264 => {
                input.raw.pop();
            }
            KEY_ENTER | 10 => {
                if !input_selected {
                    match selected_button {
                        0 => {
                            return Err(())
                        }
                        1 => {
                            return Ok(input.raw)
                        }
                        _ => {}
                    }
                }
            }
            _ => { }
        }
    }
}
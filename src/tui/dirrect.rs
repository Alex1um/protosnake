
use ncurses::*;
use crate::{tui::{err::print_error, modal::show_modal}, old::client::Client, snakes::snakes::NodeRole};

pub struct IpInput<'a> {
    pub name: &'a str,
    raw: String,
}

impl IpInput<'_> {
    pub fn new<'a>(name: &'a str) -> IpInput<'a> {
        IpInput { name, raw: String::new() }
    }
    
    pub fn default<'a>(name: &'a str, value: &str) -> IpInput<'a> {
        IpInput { name, raw: String::from(value) }
    }

}

pub fn show_connect_dialog(player_name: &str) -> Option<Client> {
    let mut inputs = vec![IpInput::new("Direct connect"), IpInput::default("Game name", "Snake game")];
    let mut selected = 0;
    let buttons_row = 2;
    let len = 3;

    const INPUT_PAIR: i16 = 1;
    const INPUT_SELECTED_PAIR: i16 = 2;
    const SUMBMIT_PAIR: i16 = 3;
    const CANCEL_PAIR: i16 = 4;
    start_color();
    init_pair(INPUT_PAIR, COLOR_WHITE, COLOR_BLACK | 0b1000);
    init_pair(INPUT_SELECTED_PAIR, COLOR_WHITE, COLOR_BLUE);
    init_pair(SUMBMIT_PAIR, COLOR_BLACK, COLOR_GREEN | 0b1000);
    init_pair(CANCEL_PAIR, COLOR_BLACK, COLOR_RED | 0b1000);
    let mut selected_button = 1;
    
    loop {
        clear();
        for (i, input) in inputs.iter().enumerate() {
            addstr(input.name);
            addstr("\n");
            if i == selected {
                attron(COLOR_PAIR(INPUT_SELECTED_PAIR));
                addstr(&format!("{:_<20}", input.raw));
                attroff(COLOR_PAIR(INPUT_SELECTED_PAIR));
            } else {
                attron(COLOR_PAIR(INPUT_PAIR));
                addstr(&format!("{:_<20}", input.raw));
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
            KEY_UP => {
                selected += len - 1;
                selected %= len;
            }
            KEY_DOWN => {
                selected += 1;
                selected %= len;
            }
            KEY_LEFT | KEY_RIGHT => {
                selected_button ^= 1;
            }
            KEY_EXIT | KEY_CANCEL | KEY_CLOSE | KEY_EOS | KEY_BREAK => {
                return None
            }
            48..=57 | 46 | 58 => { // digits | : | .
                if selected < buttons_row {
                    inputs[selected].raw.push(char::from_u32(key as u32).unwrap());
                }
            }
            65..=90 | 97..=122 | 32 => { // A-Z | a-z
                if selected == 1 {
                    inputs[selected].raw.push(char::from_u32(key as u32).unwrap());
                }
            }
            KEY_BACKSPACE | 264 => {
                if selected < buttons_row {
                    inputs[selected].raw.pop();
                }
            }
            KEY_ENTER | 10 => {
                if selected == buttons_row {
                    match selected_button {
                        0 => {
                            return None
                        }
                        1 => {
                            let role = match show_modal("Select role", vec!["Cancel", "Player", "Viewer"]) {
                                "Player" => {
                                    NodeRole::NORMAL
                                }
                                "Viewer" => {
                                    NodeRole::VIEWER
                                }
                                _ => {
                                    continue;
                                }
                            };
                            match Client::join(&inputs[0].raw, &inputs[1].raw, player_name, role) {
                                Ok(client) => return Some(client),
                                Err(e) => {
                                    print_error(e);
                                    return None;
                                },
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => { }
        }
    }
}
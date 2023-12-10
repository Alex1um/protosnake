use std::fmt::{Display, Formatter};
use ncurses::*;

pub fn show_menu<'a, 'b>(options: Vec<&'a str>, player_name: &'b mut String) -> Result<&'a str, ()> {
    let mut selected = 0;
    const INPUT_PAIR: i16 = 1;
    const INPUT_SELECTED_PAIR: i16 = 2;
    start_color();
    init_pair(INPUT_PAIR, COLOR_WHITE, COLOR_BLACK | 0b1000);
    init_pair(INPUT_SELECTED_PAIR, COLOR_WHITE, COLOR_BLUE);
    let NAME_ROW = 0;
    let len = options.len() + 1;
    loop {
        clear();
        if selected == NAME_ROW {
            attron(COLOR_PAIR(INPUT_SELECTED_PAIR));
            addstr(player_name);
            attroff(COLOR_PAIR(INPUT_SELECTED_PAIR));
        } else {
            attron(COLOR_PAIR(INPUT_PAIR));
            addstr(player_name);
            attroff(COLOR_PAIR(INPUT_PAIR));
        }
        addstr("\n");
        for (i, e) in options.iter().enumerate() {
            if i + 1 == selected {
                attron(COLOR_PAIR(INPUT_SELECTED_PAIR));
                addstr(e);
                attroff(COLOR_PAIR(INPUT_SELECTED_PAIR));
            } else {
                attron(COLOR_PAIR(INPUT_PAIR));
                addstr(e);
                attroff(COLOR_PAIR(INPUT_PAIR));
            }
            addstr("\n");
        }
        refresh();
        let key = getch();
        match key {
            KEY_UP => {
                selected = selected + len - 1;
            }
            KEY_DOWN => {
                selected += 1;
            }
            KEY_ENTER | 10 => {
                if selected != NAME_ROW {
                    return Ok(options[selected - 1]);
                }
            }
            KEY_BACKSPACE | 264 => {
                if selected == NAME_ROW {
                    player_name.pop();
                }
            }
            48..=57 | 65..=90 | 97..=122 | 95 => { // symbols
                if selected == NAME_ROW {
                    player_name.push(char::from_u32(key as u32).unwrap())
                }
            }
            KEY_EXIT | KEY_CANCEL | KEY_CLOSE | KEY_EOS | KEY_BREAK => {
                return Err(())
            }
            _ => {}
        }

        selected %= len;
    }
}

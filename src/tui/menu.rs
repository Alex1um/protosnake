use std::fmt::{Display, Formatter};
use ncurses::*;

pub fn show_menu(options: Vec<&str>) -> Result<&str, ()> {
    let mut selected = 0;
    let len = options.len();
    loop {
        clear();
        for (i, e) in options.iter().enumerate() {
            addstr(e);
            if i == selected {
                attron(A_BLINK());
                addstr(" <");
                attroff(A_BLINK());
            } else {

            }
            addstr("\n");
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
            KEY_ENTER | 10 => {
                return Ok(options[selected]);
            }
            KEY_EXIT | KEY_CANCEL | KEY_CLOSE | KEY_EOS | KEY_BREAK => {
                return Err(())
            }
            _ => {}
        }

        selected %= len;
    }
}

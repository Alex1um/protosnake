use std::fmt::Display;

use ncurses::*;

const WINDOW_HEIGHT: i32 = 6;
const WINDOW_WIDTH: i32 = 20;


fn create_win(start_y: i32, start_x: i32, text: String) -> WINDOW {
    let win = newwin(WINDOW_HEIGHT, WINDOW_WIDTH, start_y, start_x);
    box_(win, 0, 0);
    wprintw(win, &text);
    wrefresh(win);
    win
}

fn destroy_win(win: WINDOW) {
    let ch = ' ' as chtype;
    wborder(win, ch, ch, ch, ch, ch, ch, ch, ch);
    wrefresh(win);
    delwin(win);
}

pub fn print_error(err: impl Display) {
    /* Get the screen bounds. */
    let mut max_x = 0;
    let mut max_y = 0;
    getmaxyx(stdscr(), &mut max_y, &mut max_x);

    /* Start in the center. */
    let start_y = (max_y - WINDOW_HEIGHT) / 2;
    let start_x = (max_x - WINDOW_WIDTH) / 2;
    let win = create_win(start_y, start_x, format!("{}", err));
    loop {
        match getch() {
            KEY_ENTER | 10 => {
                destroy_win(win);
                break;
            }
            _ => {}
        }
    }
}
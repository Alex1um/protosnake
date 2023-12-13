
use ncurses::*;

const WINDOW_HEIGHT: i32 = 6;
const WINDOW_WIDTH: i32 = 20;


fn create_win(start_y: i32, start_x: i32, text: &str, options: &Vec<&'static str>, selected: usize) -> WINDOW {
    let win = newwin(WINDOW_HEIGHT, WINDOW_WIDTH, start_y, start_x);
    box_(win, 0, 0);
    start_color();
    wprintw(win, text);
    wprintw(win, "\n");
    const SUMBMIT_PAIR: i16 = 3;
    const CANCEL_PAIR: i16 = 4;
    init_pair(SUMBMIT_PAIR, COLOR_BLACK, COLOR_BLUE | 0b1000);
    init_pair(CANCEL_PAIR, COLOR_BLACK, COLOR_BLACK | 0b1000);

    for (i, option) in options.iter().enumerate() {
        if i == selected {
            attron(COLOR_PAIR(SUMBMIT_PAIR));
            wprintw(win, option);
            attroff(COLOR_PAIR(SUMBMIT_PAIR));
            wprintw(win, "\t");
        } else {
            wprintw(win, option);
            wprintw(win, "\t");
        }
    }
    wrefresh(win);
    win
}

fn update(win: WINDOW, text: &str, options: &Vec<&'static str>, selected: usize) {
    wclear(win);
    box_(win, 0, 0);
    wprintw(win, text);
    wprintw(win, "\n");
    const SUMBMIT_PAIR: i16 = 3;
    const CANCEL_PAIR: i16 = 4;
    init_pair(SUMBMIT_PAIR, COLOR_BLACK, COLOR_BLUE | 0b1000);
    init_pair(CANCEL_PAIR, COLOR_BLACK, COLOR_BLACK | 0b1000);

    for (i, option) in options.iter().enumerate() {
        if i == selected {
            wattr_on(win, COLOR_PAIR(SUMBMIT_PAIR));
            wprintw(win, option);
            wattr_off(win, COLOR_PAIR(SUMBMIT_PAIR));
            wprintw(win, "\t");
        } else {
            wprintw(win, option);
            wprintw(win, "\t");
        }
    }
    wrefresh(win);
}

fn destroy_win(win: WINDOW) {
    let ch = ' ' as chtype;
    wborder(win, ch, ch, ch, ch, ch, ch, ch, ch);
    wrefresh(win);
    delwin(win);
}

pub fn show_modal(text: &'static str, options: Vec<&'static str>) -> &'static str {
    /* Get the screen bounds. */
    let mut max_x = 0;
    let mut max_y = 0;
    getmaxyx(stdscr(), &mut max_y, &mut max_x);
    let mut selected = 0;
    let len = options.len();

    /* Start in the center. */
    let start_y = (max_y - WINDOW_HEIGHT) / 2;
    let start_x = (max_x - WINDOW_WIDTH) / 2;
    let win = create_win(start_y, start_x, text, &options, selected);
    timeout(300);
    loop {
        update(win, text, &options, selected);
        match getch() {
            KEY_ENTER | 10 => {
                destroy_win(win);
                break;
            }
            KEY_RIGHT => {
                selected += 1;
                selected %= len;
            }
            KEY_LEFT => {
                selected += len - 1;
                selected %= len;
            }
            _ => {}
        }
    }
    options[selected]
}
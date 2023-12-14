
use ncurses::*;

const WINDOW_HEIGHT: i32 = 5;
const WINDOW_WIDTH: i32 = 25;


fn create_win(start_y: i32, start_x: i32, text: &str, options: &Vec<&'static str>, selected: usize) -> (WINDOW, WINDOW) {
    let b_win = newwin(WINDOW_HEIGHT, WINDOW_WIDTH, start_y, start_x);
    box_(b_win, 0, 0);
    wrefresh(b_win);
    let win = newwin(WINDOW_HEIGHT - 2, WINDOW_WIDTH - 2, start_y + 1, start_x + 1);
    start_color();
    wrefresh(win);
    waddstr(win, text);
    waddstr(win, "\n\n");
    const SUMBMIT_PAIR: i16 = 3;
    const CANCEL_PAIR: i16 = 4;
    init_pair(SUMBMIT_PAIR, COLOR_BLACK, COLOR_BLUE | 0b1000);
    init_pair(CANCEL_PAIR, COLOR_BLACK, COLOR_BLACK | 0b1000);

    for (i, option) in options.iter().enumerate() {
        if i == selected {
            attron(COLOR_PAIR(SUMBMIT_PAIR));
            waddstr(win, option);
            attroff(COLOR_PAIR(SUMBMIT_PAIR));
            waddch(win, '\t'.into());
        } else {
            waddstr(win, option);
            waddch(win, '\t'.into());
        }
    }
    wrefresh(win);
    (b_win, win)
}

fn update(win: WINDOW, text: &str, options: &Vec<&'static str>, selected: usize) {
    wclear(win);
    waddstr(win, text);
    waddstr(win, "\n\n");
    const SUMBMIT_PAIR: i16 = 3;
    const CANCEL_PAIR: i16 = 4;
    init_pair(SUMBMIT_PAIR, COLOR_BLACK, COLOR_BLUE | 0b1000);
    init_pair(CANCEL_PAIR, COLOR_BLACK, COLOR_BLACK | 0b1000);

    for (i, option) in options.iter().enumerate() {
        if i == selected {
            wattr_on(win, COLOR_PAIR(SUMBMIT_PAIR));
            waddstr(win, option);
            wattr_off(win, COLOR_PAIR(SUMBMIT_PAIR));
            waddch(win, '\t'.into());
        } else {
            waddstr(win, option);
            waddch(win, '\t'.into());
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
    let (b_win, win) = create_win(start_y, start_x, text, &options, selected);
    timeout(300);
    loop {
        update(win, text, &options, selected);
        match getch() {
            KEY_ENTER | 10 => {
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
    destroy_win(b_win);
    destroy_win(win);
    options[selected]
}
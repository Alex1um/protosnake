
use std::fmt::Display;

use ncurses::*;

use crate::snakes::snakes::GameConfig;

const STATS_HEIGHT: i32 = 3;

fn create_win(start_x: i32, start_y: i32, width: i32, height: i32) -> WINDOW {
    let win = newwin(width, height, start_y, start_x);
    box_(win, '*'.into(), '*'.into());
    wrefresh(win);
    win
}

fn destroy_win(win: WINDOW) {
    let ch = ' ' as chtype;
    wborder(win, ch, ch, ch, ch, ch, ch, ch, ch);
    wrefresh(win);
    delwin(win);
}

struct GameInterface {
    main: WINDOW,
    stats: WINDOW,
    dbg: WINDOW,
    players: WINDOW,
}

impl GameInterface {
    pub fn new(cfg: &GameConfig) -> Self {
        let mut max_x = 0;
        let mut max_y = 0;
        getmaxyx(stdscr(), &mut max_y, &mut max_x);
        GameInterface {
            stats: create_win(0, 0, max_x + 1, STATS_HEIGHT),
            main: create_win(0, STATS_HEIGHT, cfg.width(), cfg.height()),
            players: create_win(cfg.width(), STATS_HEIGHT, max_x + 1 - cfg.width(), cfg.height()),
            dbg: create_win(0, cfg.height() + STATS_HEIGHT, max_x + 1, max_y + 1 - cfg.height() - STATS_HEIGHT),
        }
    }
}
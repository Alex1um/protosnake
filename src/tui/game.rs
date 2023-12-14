use ncurses::*;

use crate::snakes::snakes::game_state::snake::SnakeState;
use crate::snakes::snakes::{GameConfig, GamePlayer};
use crate::old::base::Game;

const STATS_HEIGHT: i32 = 3;

fn create_win(start_x: i32, start_y: i32, width: i32, height: i32) -> (WINDOW, WINDOW) {
    let border_win = newwin(height + 2, width + 2, start_y, start_x);
    box_(border_win, 0, 0);
    wrefresh(border_win);
    let win = newwin(height, width, start_y + 1, start_x + 1);
    wrefresh(win);
    (border_win, win)
}

fn destroy_win(win: (WINDOW, WINDOW)) {
    let ch = ' ' as chtype;
    wrefresh(win.1);
    delwin(win.1);
    wborder(win.0, ch, ch, ch, ch, ch, ch, ch, ch);
    wrefresh(win.0);
    delwin(win.0);
}

pub struct GameInterface {
    main: WINDOW,
    b_main: WINDOW,
    stats: WINDOW,
    b_stats: WINDOW,
    dbg: WINDOW,
    b_dbg: WINDOW,
    players: WINDOW,
    b_players: WINDOW,
}

impl GameInterface {
    pub fn new(cfg: &GameConfig) -> Self {
        let mut max_x = 0;
        let mut max_y = 0;
        getmaxyx(stdscr(), &mut max_y, &mut max_x);
        let (b_stats, stats) = create_win(0, 0, max_x - 2, STATS_HEIGHT);
        let (b_main, main) = create_win(0, STATS_HEIGHT + 2, cfg.width(), cfg.height());
        let (b_players, players) = create_win(cfg.width() + 2, STATS_HEIGHT + 2, max_x - cfg.width() - 4, cfg.height());
        let (b_dbg, dbg) = create_win(0, cfg.height() + STATS_HEIGHT + 4, max_x - 2, max_y - cfg.height() - STATS_HEIGHT - 6);
        let int = GameInterface {
            stats,
            b_stats,
            main,
            b_main,
            players,
            b_players,
            dbg,
            b_dbg,
        };
        scrollok(int.dbg, true);
        return int;
    }

    fn print_stats(&self, player: &GamePlayer) {
        wclear(self.stats);
        waddstr(self.stats, &format!("{}\t{}\t{}\t{}\n", "role", "id", "name", "score"));
        waddstr(self.stats, &format!("{:?}\t{}\t{}\t{}", player.role(), player.id(), player.name(), player.score()));
        wrefresh(self.stats);
    }

    fn print_players<'a, I>(&self, players: I, self_id: i32) 
        where I: Iterator<Item = &'a GamePlayer> {
        wclear(self.players);
        for player in players {
            if player.id() == self_id {
                wattron(self.players, A_BOLD());
                waddstr(self.players, player.name());
                waddch(self.players, '\n'.into());
                wattroff(self.players, A_BOLD());
            } else {
                waddstr(self.players, player.name());
                waddch(self.players, '\n'.into());
            }
        }
        wrefresh(self.players);
    }

    fn print_game(&self, game: &Game, self_id: i32) {

        wclear(self.main);
        const FOOD_PAIR: i16 = 4;
        init_pair(FOOD_PAIR, COLOR_GREEN, COLOR_BLACK);
        const EN_SNAK_PAIR: i16 = 5;
        init_pair(EN_SNAK_PAIR, COLOR_RED, COLOR_BLACK);
        const SELF_SNAK_PAIR: i16 = 6;
        init_pair(SELF_SNAK_PAIR, COLOR_BLUE, COLOR_BLACK);
        const ZM_SNAK_PAIR: i16 = 7;
        init_pair(ZM_SNAK_PAIR, COLOR_GREEN | 0b1000, COLOR_BLACK);
        wattron(self.main, COLOR_PAIR(FOOD_PAIR));
        for food in game.food.iter() {
            let x = food.x();
            let y = food.y();
            wmove(self.main, y, x);
            waddch(self.main, '@' as u32);
        }
        wattroff(self.main, COLOR_PAIR(FOOD_PAIR));
        for snak in game.snakes.values() {
            let snak_attr = match snak.state() {
                SnakeState::ZOMBIE => {
                    COLOR_PAIR(ZM_SNAK_PAIR)
                }
                SnakeState::ALIVE => {
                    if snak.player_id() == self_id {
                        COLOR_PAIR(SELF_SNAK_PAIR)
                    } else {
                        COLOR_PAIR(EN_SNAK_PAIR)
                    }
                }
            };
            wattron(self.main, snak_attr);
            for coord in snak.points.iter() {
                let x = coord.x();
                let y = coord.y();
                wmove(self.main, y, x);
                waddch(self.main, '#' as u32);
            }
            wattroff(self.main, snak_attr);
        }
        wmove(self.main, 0, 0);
        wrefresh(self.main);
    }

    pub fn print(&self, game: &Game, self_id: i32) {
        if let Some(player) = game.players.get(&self_id) {
            self.print_stats(player);
        }
        self.print_players(game.players.values(), self_id);
        self.print_game(game, self_id);
    }

    pub fn dbg(&self, msg: &str) {
        waddstr(self.dbg, msg);
        wrefresh(self.dbg);
    }
}

impl Drop for GameInterface {
    fn drop(&mut self) {
        destroy_win((self.b_stats, self.stats));
        destroy_win((self.b_players, self.players));
        destroy_win((self.b_main, self.main));
        destroy_win((self.b_dbg, self.dbg));
    }
}
use ncurses::*;
// mod snakes;
mod tui;
mod old;
mod snakes;
use old::client::Client;
use snakes::snakes::GameConfig;
use tui::{menu, config};
use config::NumInput;
use old::server::*;
use tui::browse::browse;
use tui::dirrect::show_connect_dialog;
use tui::err::print_error;

fn main() {

    initscr();
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
    // raw();
    keypad(stdscr(), true);
    let mut player_name = "Player".to_owned();
    loop {
        if let Ok(option) = menu::show_menu(vec!["Start", "Server list", "Dirrect connect", "Exit"], &mut player_name) {
            match option {
                "Start" => {
                    let mut options = vec![
                        NumInput::str_default("Server Name", "Snake game"),
                        NumInput::new("width"),
                        NumInput::new("height"),
                        NumInput::new("max food"),
                        NumInput::new("state delay ms"),
                        ];
                    if let Ok(_) = config::show_menu_config(&mut options) {
                        let mut cfg = GameConfig::new();
                        cfg.set_width(options[1].value);
                        cfg.set_height(options[2].value);
                        cfg.set_food_static(options[3].value);
                        cfg.set_state_delay_ms(options[4].value);
                        let mut srv = Server::new(cfg, options[0].raw.clone());
                        srv.run(&player_name);
                    }
                }
                "Server list" => {
                    if let Some(mut client) = browse(&player_name) {
                        client.play();
                    }
                },
                "Dirrect connect" => {
                    if let Some(mut client) = show_connect_dialog(&player_name) {
                        client.play();
                    }
                }
                "Exit" => {
                    break;
                }
                _ => {
                    break;
                }
            }
        }
    }
    endwin();

}

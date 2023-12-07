#![feature(variant_count)]

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

    if let Ok(option) = menu::show_menu(vec!["Start", "Server list", "Dirrect connect", "Exit"]) {
        match option {
            "Start" => {
                let mut options = vec![
                    NumInput::new("width"),
                    NumInput::new("height"),
                    NumInput::new("max food"),
                    NumInput::new("state delay ms"),
                    ];
                if let Ok(_) = config::show_menu_config(&mut options) {
                    let mut cfg = GameConfig::new();
                    cfg.set_width(options[0].value.expect("width"));
                    cfg.set_height(options[1].value.expect("height"));
                    cfg.set_food_static(options[2].value.expect("food"));
                    cfg.set_state_delay_ms(options[3].value.expect("state delay"));
                    let mut srv = Server::new(cfg, "Snake game".to_owned());
                    srv.run();
                }
            }
            "Server list" => {
                browse();
            },
            "Dirrect connect" => {
                if let Ok(ip) = show_connect_dialog() {
                    if !ip.is_empty() {
                        let rs =  
                        match Client::join(&ip, "Snake game", "player") {
                            Ok(mut client) => {
                                client.play();
                            }
                            Err(err) => {
                                print_error(format!("Failed to connect: {}", err));
                            }
                        };
                    }
                }
            }
            _ => {}
        }
    }

    endwin();

}

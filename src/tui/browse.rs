use std::{collections::{HashMap}, net::{SocketAddr, UdpSocket, Ipv4Addr}, time::Instant};

use crate::{old::{client::Client}, snakes::snakes::{GameMessage, game_message::{self}, NodeRole}, tui::{err::print_error, modal::show_modal}};

use ncurses::*;
use protobuf::Message;
use std::hash::Hash;

struct GameOption {
    name: String,
    addr: SocketAddr,
}

impl Hash for GameOption {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.addr.hash(state);
        self.name.hash(state);
    }
}

impl PartialEq for GameOption {
    fn eq(&self, other: &Self) -> bool {
        self.addr == other.addr && self.name == other.name
    }
}

impl Eq for GameOption {}

pub fn browse(player_name: &str) -> Option<Client> {
    let multicast_receiver = UdpSocket::bind("0.0.0.0:9192").expect("successful bind");
    multicast_receiver
        .join_multicast_v4(&Ipv4Addr::new(239, 192, 0, 4), &Ipv4Addr::new(0, 0, 0, 0))
        .expect("Successful join");
    let mut server_list = HashMap::<GameOption, Instant>::new();
    let mut buf = [0;1024];

    let mut selected = 0;
    const SERVER_PAIR: i16 = 1;
    const SERVER_SELECTED_PAIR: i16 = 2;
    timeout(300);
    start_color();
    init_pair(SERVER_PAIR, COLOR_WHITE, COLOR_BLACK | 0b1000);
    init_pair(SERVER_SELECTED_PAIR, COLOR_WHITE, COLOR_BLUE);

    loop {
        clear();
        if let Ok((len, addr)) = multicast_receiver.recv_from(&mut buf) {
            if let Ok(msg) = GameMessage::parse_from_bytes(&buf[..len]) {
                if let Some(tpe) = msg.Type {
                    match tpe {
                        game_message::Type::Announcement(an) => {
                            for game in an.games {
                                if game.can_join() {
                                    server_list.insert(GameOption { name: game.game_name.or(Some("Annonymous".to_owned())).unwrap(), addr: addr.clone()}, Instant::now());
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        let mut i = 0;
        server_list.retain(|game, ins| {
            if ins.elapsed().as_secs_f32() > 1.0f32 {
                if selected >= i {
                    selected -= 1;
                }
                return false;
            }
            if i == selected {
                attron(COLOR_PAIR(SERVER_SELECTED_PAIR));
                addstr(&format!("{} {}", game.name, game.addr.to_string()));
                attroff(COLOR_PAIR(SERVER_SELECTED_PAIR));
            } else {
                attron(COLOR_PAIR(SERVER_PAIR));
                addstr(&format!("{} {}", game.name, game.addr.to_string()));
                attroff(COLOR_PAIR(SERVER_PAIR));
            }
            addstr("\n");

            i += 1;
            true
        });

        let len = server_list.len();
        refresh();

        let key = getch();
        match key {
            KEY_UP | 119 => {
                selected = selected + len - 1;
            }
            KEY_DOWN | 115 => {
                selected += 1;
            }
            KEY_EXIT | KEY_CANCEL | KEY_CLOSE | KEY_EOS | KEY_BREAK => {
                return None;
            }
            KEY_ENTER | 10 => {
                if len == 0 {
                    return None;
                }
                let key = server_list.keys().nth(selected).expect("selected variant exists");
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
                match Client::join( key.addr.clone(), &key.name, player_name, role) {
                    Ok(cl) => return Some(cl),
                    Err(e) => {
                        print_error(format!("Failed to connect to server: {:?}", e));
                    }
                }
            }
            _ => { }
        }
        if len > 0 {
            selected %= len;
        }
    }
}
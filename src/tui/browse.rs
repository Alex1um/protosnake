use std::{collections::HashMap, net::SocketAddr};

use crate::{old::{sockets::Sockets, client::Client}, snakes::snakes::{GameAnnouncement, GameMessage, game_message::{AnnouncementMsg, self}}};
use ncurses::*;
use protobuf::Message;

pub fn browse() -> Option<Client> {
    let sockets = Sockets::new3(true);
    let mut server_list = HashMap::<SocketAddr, AnnouncementMsg>::new();
    let mut buf = [0;1024];

    let mut selected = 0;
    const SERVER_PAIR: i16 = 1;
    const SERVER_SELECTED_PAIR: i16 = 2;
    start_color();
    init_pair(SERVER_PAIR, COLOR_WHITE, COLOR_BLACK | 0b1000);
    init_pair(SERVER_SELECTED_PAIR, COLOR_WHITE, COLOR_BLUE);

    loop {
        clear();
        if let Ok((len, addr)) = sockets.multicast_receiver.recv_from(&mut buf) {
            if let Ok(msg) = GameMessage::parse_from_bytes(&buf) {
                if let Some(tpe) = msg.Type {
                    match tpe {
                        game_message::Type::Announcement(an) => {
                            server_list.insert(addr, an).expect("new game insert");
                        }
                        _ => {}
                    }
                }
            }
        }
        for (i, (addr, msg)) in server_list.iter().enumerate() {
            if i == selected {
                attron(COLOR_PAIR(SERVER_SELECTED_PAIR));
                addstr(&format!("{} {}", addr.to_string(), msg.to_string()));
                attroff(COLOR_PAIR(SERVER_SELECTED_PAIR));
            } else {
                attron(COLOR_PAIR(SERVER_PAIR));
                addstr(&format!("{} {}", addr.to_string(), msg.to_string()));
                attroff(COLOR_PAIR(SERVER_PAIR));
            }
            addstr("\n");
        }
        refresh();
    }
    None
}
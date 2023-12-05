use std::net::{ToSocketAddrs, SocketAddr};

use protobuf::Message;

use crate::snakes::snakes::{GameConfig, game_message::{JoinMsg, self, StateMsg, SteerMsg, DiscoverMsg}, PlayerType, NodeRole, GameMessage, game_state::Coord, Direction};

use super::{base::Game, sockets::Sockets};

use anyhow::{Result, bail};

use std::thread;
        
use ncurses::*;

pub struct Client {
    game: Game,
    player_name: String,
    sockets: Sockets,
    id: i32,
}

impl Client {

    pub fn new(config: GameConfig, player_name: String, player_id: i32) -> Self {
        Client {
            game: Game::new(config),
            player_name,
            sockets: Sockets::new2(true),
            id: player_id,
        }
    }

    fn recv_game_message(&self) -> Result<GameMessage> {
        let mut buf = [0u8; 1024];
        let (len, addr) = self.sockets.socket.recv_from(&mut buf)?;
        Ok(GameMessage::parse_from_bytes(&buf)?)
    }

    fn wait_announcement(sockets: &mut Sockets, game_name: &str) -> Result<GameConfig> {
        let mut buf = [0u8; 1024];
        let len = sockets.socket.recv(&mut buf)?;
        let gm = &GameMessage::parse_from_bytes(&buf)?;
        if let Some(ref r#type) = gm.Type {
            match r#type {
                game_message::Type::Announcement(ann) => {
                    for game in ann.games.iter() {
                        if game.game_name() == game_name {
                            return Ok((*(game.config)).clone());
                        }
                    }
                    bail!("No game with name {} was found", game_name);
                }
                _ => {
                    bail!("Unexpected message from server");
                }
            }
        } else {
            bail!("wrong message type were received: {:?}", gm.Type)
        }
    }

    fn wait_ack(sockets: &mut Sockets) -> Result<i32> {
        let mut buf = [0u8; 1024];
        let len = sockets.socket.recv(&mut buf)?;
        let gm = &GameMessage::parse_from_bytes(&buf)?;
        if let Some(ref r#type) = gm.Type {
            match r#type {
                game_message::Type::Ack(ack) => {
                    return Ok(gm.receiver_id())
                }
                _ => {
                    bail!("Unexpected message from server");
                }
            }
        } else {
            bail!("wrong message type were received: {:?}", gm.Type)
        }

    }

    pub fn join<T>(addr: T, game_name: &str, player_name: &str) -> Result<Self>
    where T: ToSocketAddrs {
        let mut sockets = Sockets::new2(false);
        sockets.socket.connect(&addr)?;
        let mut msg = GameMessage::new();
        msg.set_discover(DiscoverMsg::new());
        sockets.socket.send(&msg.write_to_bytes()?)?;
        let config = Self::wait_announcement(&mut sockets, game_name)?;

        let mut join_msg = JoinMsg::new();
        join_msg.set_player_type(PlayerType::HUMAN);
        join_msg.set_requested_role(NodeRole::NORMAL);
        join_msg.set_game_name(game_name.to_string());
        join_msg.set_player_name(player_name.to_string());
        let mut msg = GameMessage::new();
        msg.set_join(join_msg);
        let bytes = msg.write_to_bytes()?;
        sockets.socket.send(&bytes)?;

        let pid = Self::wait_ack(&mut sockets)?;
        sockets.socket.set_nonblocking(true);
        sockets.multicast_receiver.set_nonblocking(true);
        Ok(Client {
            game: Game::new(config),
            player_name: player_name.to_owned(),
            sockets,
            id: pid,
        })
        
    }

    fn print(&self) {
        clear();
        const FOOD_PAIR: i16 = 4;
        init_pair(FOOD_PAIR, COLOR_GREEN, COLOR_BLACK);
        const EN_SNAK_PAIR: i16 = 5;
        init_pair(EN_SNAK_PAIR, COLOR_RED, COLOR_BLACK);
        const SELF_SNAK_PAIR: i16 = 6;
        init_pair(SELF_SNAK_PAIR, COLOR_BLUE, COLOR_BLACK);
        attron(COLOR_PAIR(FOOD_PAIR));
        for food in self.game.food.iter() {
            let x = food.x();
            let y = food.y();
            mv(y, x);
            addch('@' as u32);
        }
        attroff(COLOR_PAIR(FOOD_PAIR));
        attron(COLOR_PAIR(EN_SNAK_PAIR));
        for snak in self.game.snakes.values() {
            for coord in snak.points.iter() {
                let x = coord.x();
                let y = coord.y();
                mv(y, x);
                addch('#' as u32);
            }
        }
        attroff(COLOR_PAIR(EN_SNAK_PAIR));
        if let Some(snak) = self.game.snakes.get(&self.id) {
            attron(COLOR_PAIR(SELF_SNAK_PAIR));
            for coord in snak.points.iter() {
                let x = coord.x();
                let y = coord.y();
                mv(y, x);
                addch('#' as u32);
            }

            attroff(COLOR_PAIR(SELF_SNAK_PAIR));
        }
        mv(0, 0);
        refresh();
    }

    fn send_steer(&self, dir: Direction, addr: SocketAddr) {
        let mut steer = SteerMsg::new();
        steer.set_direction(dir);
        let mut gm = GameMessage::new();
        gm.set_sender_id(self.id);
        gm.set_msg_seq(0);
        gm.set_steer(steer);
        if let Err(e) = self.sockets.socket.send_to(&gm.write_to_bytes().expect("bytes"), addr) {
            panic!("{:?}", e);
        }
    }

    pub fn prepare(&mut self) {
        self.sockets.socket.set_nonblocking(true);
        timeout(300);
    }

    pub fn action(&mut self) {
        let mut buf = [0u8; 1024];
        static mut cur_addr: Option<SocketAddr> = None;
        if let Ok((len, addr)) = self.sockets.socket.recv_from(&mut buf) {
            if let Ok(gm) = GameMessage::parse_from_bytes(&buf[..len]) {
                if let Some(tpe) = gm.Type {
                    match tpe {
                        game_message::Type::State(state) => {
                            self.game.apply_state(state.state);
                            self.print();
                        }
                        _ => {}
                    }
                }
            }
            match getch() {
                KEY_LEFT => {
                    self.send_steer(Direction::LEFT, addr);
                }
                KEY_RIGHT => {
                    self.send_steer(Direction::RIGHT, addr);
                }
                KEY_UP => {
                    self.send_steer(Direction::UP, addr);
                }
                KEY_DOWN => {
                    self.send_steer(Direction::DOWN, addr);
                }
                ERR => {

                }
                _  => {
                }
            }
        }
    }

    pub fn play(&mut self) {
        self.prepare();
        loop {
            self.action();
        }
    }


}

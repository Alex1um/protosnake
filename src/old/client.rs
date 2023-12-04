use std::net::{ToSocketAddrs, SocketAddr};

use protobuf::Message;

use crate::snakes::snakes::{GameConfig, game_message::{JoinMsg, self, StateMsg, SteerMsg}, PlayerType, NodeRole, GameMessage, game_state::Coord, Direction};

use super::{base::Game, sockets::Sockets};

use anyhow::{Result, bail};

pub struct Client {
    game: Game,
    player_name: String,
    sockets: Sockets,
    id: Option<i32>,
}

impl Client {

    pub fn new(config: GameConfig, player_name: String) -> Self {
        Client {
            game: Game::new(config),
            player_name,
            sockets: Sockets::new(false),
            id: None,
        }
    }

    fn recv_game_message(&self) -> Result<GameMessage> {
        let mut buf = [0u8; 1024];
        let (len, addr) = self.sockets.socket.recv_from(&mut buf)?;
        Ok(GameMessage::parse_from_bytes(&buf)?)
    }

    pub fn join<T>(&mut self, addr: T, game_name: &str) -> Result<()>
    where T: ToSocketAddrs {
        let mut join_msg = JoinMsg::new();
        join_msg.set_player_type(PlayerType::HUMAN);
        join_msg.set_requested_role(NodeRole::NORMAL);
        join_msg.set_game_name(game_name.to_string());
        join_msg.set_player_name(self.player_name.to_string());
        let mut msg = GameMessage::new();
        msg.set_join(join_msg);
        let bytes = msg.write_to_bytes()?;
        self.sockets.socket.send_to(&bytes, &addr)?;
        let gm = self.recv_game_message()?;
        if let Some(ref r#type) = gm.Type {
            match r#type {
                game_message::Type::Ack(ack) => {
                    self.id = Some(gm.receiver_id());
                    self.sockets.socket.connect(addr);
                    return Ok(());
                }
                _ => {
                    bail!("Unexpected message from server");
                }
            }
        }
        bail!("Wrong game message structure")
    }

    fn print(&self) {
        use ncurses::*;
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
        if let Some(id) = &self.id {
            if let Some(snak) = self.game.snakes.get(id) {
                attron(COLOR_PAIR(SELF_SNAK_PAIR));
                for coord in snak.points.iter() {
                    let x = coord.x();
                    let y = coord.y();
                    mv(y, x);
                    addch('#' as u32);
                }

                attroff(COLOR_PAIR(SELF_SNAK_PAIR));
            }
        }
        mv(0, 0);
        refresh();
    }

    fn send_steer(&self, dir: Direction, addr: &SocketAddr) {
        let mut steer = SteerMsg::new();
        steer.set_direction(dir);
        let mut gm = GameMessage::new();
        gm.set_steer(steer);
        self.sockets.socket.send(&gm.write_to_bytes().expect("bytes"));
    }

    fn play(&mut self) {
        use ncurses::*;

        let mut buf = [0u8; 1024];
        self.sockets.socket.set_nonblocking(true);
        timeout(300);
        loop {
            if let Ok((_, addr)) = self.sockets.socket.recv_from(&mut buf) {
                if let Ok(gm) = GameMessage::parse_from_bytes(&buf) {
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
                        self.send_steer(Direction::LEFT, &addr);
                    }
                    KEY_RIGHT => {
                        self.send_steer(Direction::RIGHT, &addr);
                    }
                    KEY_UP => {
                        self.send_steer(Direction::UP, &addr);
                    }
                    KEY_DOWN => {
                        self.send_steer(Direction::DOWN, &addr);
                    }
                    ERR => {

                    }
                    _ => {}
                }
            }
        }
    }


}

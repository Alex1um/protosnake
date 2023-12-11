use std::{net::{ToSocketAddrs, SocketAddr}, time::Instant, collections::HashMap};

use protobuf::Message;

use crate::snakes::snakes::{GameConfig, game_message::{JoinMsg, self, StateMsg, SteerMsg, DiscoverMsg, RoleChangeMsg}, PlayerType, NodeRole, GameMessage, game_state::{Coord, snake::SnakeState}, Direction};

use super::{base::Game, sockets::Sockets};

use anyhow::{Result, bail};

use ncurses::*;

struct PendingMsg {
    msg: GameMessage,
    send_time: Option<Instant>,
}

impl PendingMsg {
    pub fn new(msg: GameMessage) -> Self {
        PendingMsg {
            msg,
            send_time: None,
        }
    }
}

pub struct Client {
    game: Game,
    player_name: String,
    sockets: Sockets,
    id: i32,
    role: NodeRole,
    seq: i64,
    last_mesg: Instant,
    pending_msgs: HashMap<i64, PendingMsg>
}

impl Client {

    pub fn new(config: GameConfig, player_name: String, player_id: i32, role: NodeRole) -> Self {
        Client {
            game: Game::new(config),
            player_name,
            sockets: Sockets::new2(true),
            id: player_id,
            role,
            seq: 0,
            last_mesg: Instant::now(),
            pending_msgs: HashMap::new(),
        }
    }

    fn wait_announcement(sockets: &mut Sockets, game_name: &str) -> Result<GameConfig> {
        let mut buf = [0u8; 1024];
        let len = sockets.socket.recv(&mut buf)?;
        let gm = GameMessage::parse_from_bytes(&buf[..len])?;
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

    pub fn join<T>(addr: T, game_name: &str, player_name: &str, role: NodeRole) -> Result<Self>
    where T: ToSocketAddrs {

        fn wait_ack(sockets: &mut Sockets) -> Result<i32> {
            let mut buf = [0u8; 1024];
            let len = sockets.socket.recv(&mut buf)?;
            let gm = &GameMessage::parse_from_bytes(&buf[..len])?;
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

        let mut sockets = Sockets::new3(false);
        sockets.socket.connect(&addr)?;
        let mut msg = GameMessage::new();
        msg.set_discover(DiscoverMsg::new());
        msg.set_msg_seq(0);
        sockets.socket.send(&msg.write_to_bytes()?)?;
        let config = Self::wait_announcement(&mut sockets, game_name)?;

        let mut join_msg = JoinMsg::new();
        join_msg.set_player_type(PlayerType::HUMAN);
        join_msg.set_requested_role(role);
        join_msg.set_game_name(game_name.to_string());
        join_msg.set_player_name(player_name.to_string());
        let mut msg = GameMessage::new();
        msg.set_join(join_msg);
        msg.set_msg_seq(0);
        let bytes = msg.write_to_bytes()?;
        sockets.socket.send(&bytes)?;

        let pid = wait_ack(&mut sockets)?;
        sockets.socket.set_nonblocking(true);
        Ok(Client {
            game: Game::new(config),
            player_name: player_name.to_owned(),
            sockets,
            id: pid,
            role,
            seq: 0,
            last_mesg: Instant::now(),
            pending_msgs: HashMap::new(),
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
        const ZM_SNAK_PAIR: i16 = 7;
        init_pair(ZM_SNAK_PAIR, COLOR_GREEN | 0b1000, COLOR_BLACK);
        attron(COLOR_PAIR(FOOD_PAIR));
        for food in self.game.food.iter() {
            let x = food.x();
            let y = food.y();
            mv(y, x);
            addch('@' as u32);
        }
        attroff(COLOR_PAIR(FOOD_PAIR));
        let mut snak_attroff = COLOR_PAIR(EN_SNAK_PAIR);
        for snak in self.game.snakes.values() {
            match snak.state() {
                SnakeState::ZOMBIE => {
                    attron(COLOR_PAIR(ZM_SNAK_PAIR));
                    snak_attroff = COLOR_PAIR(ZM_SNAK_PAIR)
                }
                SnakeState::ALIVE => {
                    snak_attroff = COLOR_PAIR(EN_SNAK_PAIR);
                    attron(COLOR_PAIR(EN_SNAK_PAIR));
                }
            }
            for coord in snak.points.iter() {
                let x = coord.x();
                let y = coord.y();
                mv(y, x);
                addch('#' as u32);
            }
            attroff(snak_attroff);
        }
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

    fn send_steer(&mut self, dir: Direction) {
        let mut steer = SteerMsg::new();
        steer.set_direction(dir);
        let mut gm = GameMessage::new();
        gm.set_sender_id(self.id);
        gm.set_msg_seq(self.seq);
        gm.set_steer(steer);
        self.pending_msgs.insert(self.seq, PendingMsg::new(gm));
        self.seq += 1;
    }

    fn send_change_player(&mut self) {
        let mut chnge = RoleChangeMsg::new();
        chnge.set_sender_role(NodeRole::NORMAL);
        let mut gm = GameMessage::new();
        gm.set_sender_id(self.id);
        gm.set_msg_seq(self.seq);
        gm.set_role_change(chnge);
        self.pending_msgs.insert(self.seq, PendingMsg::new(gm));
        self.seq += 1;
    }

    fn send_change_viewer(&mut self) {
        let mut chnge = RoleChangeMsg::new();
        chnge.set_sender_role(NodeRole::VIEWER);
        let mut gm = GameMessage::new();
        gm.set_sender_id(self.id);
        gm.set_msg_seq(self.seq);
        gm.set_role_change(chnge);
        self.pending_msgs.insert(self.seq, PendingMsg::new(gm));
        self.seq += 1;
    }

    pub fn prepare(&mut self) {
        self.sockets.socket.set_nonblocking(true);
        timeout(300);
    }

    fn check_pending(&mut self, addr: SocketAddr) {
        let delay = self.game.config.state_delay_ms() as u128;
        let now = Instant::now();
        for (_, v) in self.pending_msgs.iter() {
            match v.send_time {
                None => {
                    self.sockets.socket.send_to(&v.msg.write_to_bytes().expect("can write gamemessage to bytes"), addr);
                }
                Some(t) => {
                    if (now - t).as_millis() > delay {
                        self.sockets.socket.send_to(&v.msg.write_to_bytes().expect("can write gamemessage to bytes"), addr);
                    }
                }
            }
        }
    }

    fn process_ack(&mut self, seq: i64) {
        self.pending_msgs.remove(&seq);
    }

    pub fn action(&mut self) -> bool {
        let mut buf = [0u8; 1024];
        static mut cur_addr: Option<SocketAddr> = None;
        if let Ok((len, addr)) = self.sockets.socket.recv_from(&mut buf) {
            if len == 0 {
                return false;
            }
            if let Ok(gm) = &GameMessage::parse_from_bytes(&buf[..len]) {
                if let Some(tpe) = &gm.Type {
                    match tpe {
                        game_message::Type::State(state) => {
                            if let Some(state) = state.state.as_ref() {
                                self.game.apply_state(state);
                                self.print();
                            }
                        }
                        game_message::Type::Ack(_) => {
                            self.process_ack(gm.msg_seq());
                        }
                        _ => {}
                    }
                }
            }
            match getch() {
                KEY_LEFT => {
                    self.send_steer(Direction::LEFT);
                }
                KEY_RIGHT => {
                    self.send_steer(Direction::RIGHT);
                }
                KEY_UP => {
                    self.send_steer(Direction::UP);
                }
                KEY_DOWN => {
                    self.send_steer(Direction::DOWN);
                }
                KEY_ENTER | 10 => {
                    self.send_change_player();
                }
                KEY_BACKSPACE => {
                    self.send_change_viewer();
                }
                ERR => {

                }
                _  => {
                }
            }
            self.check_pending(addr);
        }
        true
    }

    pub fn play(&mut self) {
        self.prepare();
        loop {
            if !self.action() {
                break
            }
        }
    }


}

use std::collections::HashMap;
use std::io;
use std::net::{SocketAddr, ToSocketAddrs, SocketAddrV4};
use std::time::Instant;
use protobuf::{Message, MessageField};
use super::client::Client;
use super::sockets::Sockets;
use crate::snakes::snakes::game_state::Snake;
use crate::snakes::snakes::game_state::snake::SnakeState;
use crate::snakes::snakes::{GameAnnouncement, GameConfig, GameMessage, GamePlayer, GamePlayers, game_message, GameState, NodeRole, Direction};
use crate::snakes::snakes::game_message::{AnnouncementMsg, DiscoverMsg, PingMsg, AckMsg, StateMsg, ErrorMsg};

use super::base::{Game, WorldCell};

pub struct Server {
    game: Game,
    sockets: Sockets,
    name: String,
    seq: i64,
    id: i32,
    multicast_instant: Instant,
}

impl Server {
    pub fn new(config: GameConfig, name: String) -> Self {
        Server {
            game: Game::new(config),
            sockets: Sockets::new(true),
            name: name,
            seq: 0,
            id: 0,
            multicast_instant: Instant::now(),
        }
    }

    fn get_announcement(&mut self) -> GameMessage {
        let mut current_game_info = GameAnnouncement::new();
        current_game_info.game_name = Some(self.name.clone());
        let mut players = GamePlayers::new();
        players.players = self.game.players.clone();
        current_game_info.players = MessageField::some(players);
        current_game_info.config = MessageField::some(self.game.config.clone());
        current_game_info.set_can_join(true);
        let mut announcement = AnnouncementMsg::new();
        announcement.games.push(current_game_info);
        let mut game_message = GameMessage::new();
        game_message.set_msg_seq(self.seq);
        self.seq += 1;
        // game_message.set_receiver_id(0);
        game_message.set_sender_id(self.id);
        game_message.set_announcement(announcement);
        game_message
    }

    fn send_multicast(&mut self) {

        let announcement = self.get_announcement();
        
    }
    
    pub fn get_state(&mut self) {
        let mut state = GameState::new();
        state.foods.clone_from(&self.game.food);
        state.snakes = self.game.snakes.values().cloned().collect();
        let mut players = GamePlayers::new();
        players.players = self.game.players.clone();
        state.players = Some(players).into();
        state.set_state_order(0);
        let mut state_msg = StateMsg::new();
        state_msg.state = Some(state).into();
        let mut msg = GameMessage::new();
        msg.set_state(state_msg);
        msg.set_msg_seq(0);
        msg.set_sender_id(0);
        msg.set_receiver_id(0);
        let bytes = msg.write_to_bytes().expect("written state");
        for player in self.game.players.iter() {
            self.sockets.socket.send_to(&bytes, player.ip_address());
        }
    }

    fn get_player_by_ip_mut(&mut self, addr: &SocketAddr) -> Option<&mut GamePlayer> {
        let str_addr = addr.to_string();
        for e in self.game.players.iter_mut() {
            if e.ip_address() == str_addr {
                return Some(e);
            }
        }
        return None;
    }
    
    fn get_player_by_id_mut(&mut self, id: i32) -> Option<&mut GamePlayer> {
        for e in self.game.players.iter_mut() {
            if e.id() == id {
                return Some(e);
            }
        }
        return None;
    }
    
    fn get_player_by_id(&self, id: i32) -> Option<&GamePlayer> {
        for e in self.game.players.iter() {
            if e.id() == id {
                return Some(e);
            }
        }
        return None;
    }

    fn get_player_by_ip(&self, addr: &SocketAddr) -> Option<&GamePlayer> {
        let str_addr = addr.to_string();
        for e in self.game.players.iter() {
            if e.ip_address() == str_addr {
                return Some(e);
            }
        }
        return None;
    }

    
    fn send_ack(&self, seq: i64, receiver_id: Option<i32>, addr: &SocketAddr) {
        let mut game_msg = GameMessage::new();
        game_msg.set_ack(AckMsg::new());
        game_msg.set_msg_seq(seq);
        if let Some(pid) = receiver_id {
            game_msg.set_receiver_id(pid);
        }
        self.sockets.socket.send_to(&game_msg.write_to_bytes().expect("written ack"), addr).expect("send ack");
    }

    fn receive_message(&mut self) {
        let mut buf = [0u8; 1024];
        if let Ok((len, addr)) = self.sockets.socket.recv_from(&mut buf) {
            if let Ok(msg) = GameMessage::parse_from_bytes(&buf[..len]) {
                if let Some(t) = &msg.Type {
                    match t {
                        game_message::Type::Ping(ping_msg) => {
                            self.send_ack(msg.msg_seq(), None, &addr);
                        }
                        game_message::Type::Ack(ack) => {

                        }
                        game_message::Type::Announcement(announcement) => {

                        }
                        game_message::Type::Discover(discover) => {
                            let announcement = self.get_announcement();
                            if let Ok(bytes) = announcement.write_to_bytes() {
                                let res = self.sockets.socket.send_to(&bytes, &addr);
                            }
                        }
                        game_message::Type::Error(error) => {
                            self.send_ack(msg.msg_seq(), None, &addr);
                        }
                        game_message::Type::Join(join) => {
                            let mut player = GamePlayer::new();
                            player.set_name(join.player_name().to_string());
                            player.set_ip_address(addr.to_string());
                            player.set_role(join.requested_role()); // TODO: change
                            player.set_type(join.player_type());
                            player.set_id(self.id + 1);
                            self.id += 1;
                            player.set_score(0);
                            if join.requested_role() != NodeRole::VIEWER {
                                if let Some((head, tail, dir)) = self.game.get_free_coord5x5() {
                                    self.game.world[head.y() as usize][head.x() as usize] = WorldCell::Snake;
                                    self.game.world[tail.y() as usize][tail.x() as usize] = WorldCell::Snake;
                                    let mut snake = Snake::new();
                                    snake.points.push(tail);
                                    snake.points.push(head);
                                    snake.set_head_direction(dir);
                                    snake.set_state(SnakeState::ALIVE);
                                    snake.set_player_id(player.id());
                                    self.game.snakes.insert(player.id(), snake);
                                    self.game.players.push(player);
                                } else {
                                    let mut error = ErrorMsg::new();
                                    error.set_error_message("no available space for snake".to_string());
                                    let mut game_msg = GameMessage::new();
                                    game_msg.set_msg_seq(0);
                                    game_msg.set_error(error);
                                    if let Ok(bytes) = game_msg.write_to_bytes() {
                                        let _ = self.sockets.socket.send_to(&bytes, addr);
                                    }
                                }
                            } else {
                                self.game.players.push(player);
                            }
                            self.send_ack(msg.msg_seq(), None, &addr);
                        }
                        game_message::Type::RoleChange(role_change) => {
                            if let Some((old_role, id)) = self.get_player_by_ip(&addr).and_then(|ply| Some((ply.role(), ply.id()))) {
                                let new_role = role_change.sender_role();
                                if new_role != old_role {
                                    if new_role == NodeRole::VIEWER {
                                        self.game.snakes.entry(id).and_modify(|snak| snak.set_state(SnakeState::ZOMBIE));
                                        self.get_player_by_ip_mut(&addr).unwrap().set_role(new_role);
                                    } else if new_role == NodeRole::NORMAL {
                                        if let Some(snak) = self.game.snakes.get_mut(&id) {
                                            snak.set_state(SnakeState::ALIVE);
                                            self.get_player_by_ip_mut(&addr).unwrap().set_role(new_role);
                                        } else {
                                            if let Some((head, tail, dir)) = self.game.get_free_coord5x5() {
                                                self.game.world[head.y() as usize][head.x() as usize] = WorldCell::Snake;
                                                self.game.world[tail.y() as usize][tail.x() as usize] = WorldCell::Snake;
                                                let mut snake = Snake::new();
                                                snake.points.push(tail);
                                                snake.points.push(head);
                                                snake.set_head_direction(dir);
                                                snake.set_state(SnakeState::ALIVE);
                                                snake.set_player_id(id);
                                                self.game.snakes.insert(id, snake);
                                                self.get_player_by_ip_mut(&addr).unwrap().set_role(new_role);
                                            }

                                        }
                                    }
                                } 
                                
                            }
                            
                            self.send_ack(msg.msg_seq(), None, &addr);
                        }
                        game_message::Type::State(state) => {
                            self.send_ack(msg.msg_seq(), None, &addr);
                        }
                        game_message::Type::Steer(steer) => {
                            if let Some(player) = self.get_player_by_ip(&addr) {
                                let id = player.id();
                                if player.role() != NodeRole::VIEWER {
                                    if let Some(p) = self.game.snakes.get_mut(&id) {
                                        let opposite_dir = match p.head_direction() {
                                            Direction::UP => Direction::DOWN,
                                            Direction::DOWN => Direction::UP,
                                            Direction::LEFT => Direction::RIGHT,
                                            Direction::RIGHT => Direction::LEFT,
                                        };
                                        if steer.direction() != opposite_dir {
                                            p.set_head_direction(steer.direction())
                                        }
                                    }
                                }
                            }
                            self.send_ack(msg.msg_seq(), None, &addr);
                        }
                    }
                    // ack
                    // match &t {
                    //     game_message::Type::Announcement(_) | game_message::Type::Discover(_) | game_message::Type::Ack(_) => {},
                    //     _ => {
                    //         self.send_ack(msg.msg_seq(), );
                    //     }

                    // }
                }
                // timeout update
                // if let Some(p) = self.get_player_by_ip(&addr) {
                //     let id = p.id();
                //     self.time_map.insert(id, Instant::now());
                // }
            }
        }
    }

    fn announce(&mut self) {
        let announcemet = self.get_announcement();
        self.sockets.socket.send_to(&announcemet.write_to_bytes().expect("write announcement"), "239.192.0.4:9192").expect("announce send");
    }

    fn add_local_player(&mut self, name: &str) -> Client {
        let local_client = Client::new(
            self.game.config.clone(),
            String::from(name),
            0,
            NodeRole::MASTER,
            "127.0.0.1:48666".to_socket_addrs().unwrap().next().unwrap(),
        );
        let mut player = GamePlayer::new();
        player.set_name("Admin".to_owned());
        player.set_ip_address("127.0.0.1:48668".to_owned());
        player.set_score(0);
        player.set_role(NodeRole::MASTER); // TODO: change
        player.set_type(crate::snakes::snakes::PlayerType::HUMAN);
        player.set_id(0);
        if let Some((head, tail, dir)) = self.game.get_free_coord5x5() {
            self.game.world[head.y() as usize][head.x() as usize] = WorldCell::Snake;
            self.game.world[tail.y() as usize][tail.x() as usize] = WorldCell::Snake;
            let mut snake = Snake::new();
            snake.points.push(tail);
            snake.points.push(head);
            snake.set_head_direction(dir);
            snake.set_state(SnakeState::ALIVE);
            snake.set_player_id(0);
            self.game.snakes.insert(player.id(), snake);
        }
        self.game.players.push(player);
        local_client
    }

    pub fn run(&mut self, player_name: &str) {
        // endwin();
        let mut client = self.add_local_player(player_name);
        client.prepare();
        let mut game_timestamp = Instant::now();
        let mut announce_timestamp = Instant::now();
        loop {
            self.receive_message();
            if game_timestamp.elapsed().as_millis() as i32 > self.game.config.state_delay_ms() {
                self.game.update();
                self.get_state();
                game_timestamp = Instant::now();
            }
            if announce_timestamp.elapsed().as_millis() > 1000 {
                self.announce();
                announce_timestamp = Instant::now();
            }
            client.action();
        }
    }
}

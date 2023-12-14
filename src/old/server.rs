use std::collections::HashMap;
use std::io;
use std::net::{SocketAddr, ToSocketAddrs};
use std::time::Instant;
use protobuf::{Message, MessageField};
use super::client::Client;
use super::sockets::Sockets;
use crate::snakes::snakes::game_state::Snake;
use crate::snakes::snakes::game_state::snake::SnakeState;
use crate::snakes::snakes::{GameAnnouncement, GameConfig, GameMessage, GamePlayer, GamePlayers, game_message, GameState, NodeRole, Direction};
use crate::snakes::snakes::game_message::{AnnouncementMsg, AckMsg, StateMsg, ErrorMsg, RoleChangeMsg};
use anyhow::Result;

use super::base::{Game, WorldCell};

struct PendingMsg {
    msg: Vec<u8>,
    send_time: Option<Instant>,
    send_count: u32,
    addr: SocketAddr,
}

impl PendingMsg {
    pub fn new(msg: &GameMessage, addr: SocketAddr) -> Self {
        PendingMsg {
            msg: msg.write_to_bytes().expect("written msg"),
            send_time: None,
            send_count: 0,
            addr
        }
    }

    pub fn send(&mut self, sockets: &mut Sockets) -> Result<(), io::Error> {
        sockets
            .socket
            .send_to(&self.msg, self.addr)
            .and_then(|_| {
                self.send_count += 1;
                Ok(())
            })
    }
}
pub struct Server {
    game: Game,
    sockets: Sockets,
    name: String,
    seq: i64,
    id: i32,
    multicast_instant: Instant,
    game_instant: Instant,
    pending_msgs: HashMap<i64, PendingMsg>,
    player_timeout: HashMap<SocketAddr, Instant>,
    player_id: i32,
    addrs: HashMap<SocketAddr, i32>,
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
            game_instant: Instant::now(),
            pending_msgs: HashMap::new(),
            player_timeout: HashMap::new(),
            addrs: HashMap::new(),
            player_id: 1,
        }
    }

    fn get_announcement(&mut self) -> GameMessage {
        let mut current_game_info = GameAnnouncement::new();
        current_game_info.game_name = Some(self.name.clone());
        let mut players = GamePlayers::new();
        players.players = self.game.players.values().cloned().collect();
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

    fn do_multicast(&mut self) {
        let now = Instant::now();
        if (now - self.multicast_instant).as_millis() > 1000 {
            let announcement = self.get_announcement();
            self.sockets.socket.send_to(&announcement.write_to_bytes().expect("announcement bytes"), "239.192.0.4:9192").expect("multicast send");
            self.multicast_instant = now;
        }
    }

    fn get_chnge_role(&mut self, _role: NodeRole, id: i32) -> GameMessage {
        let mut gm = GameMessage::new();
        let mut rc = RoleChangeMsg::new();
        rc.set_sender_role(NodeRole::MASTER);
        rc.set_receiver_role(NodeRole::VIEWER);
        gm.set_role_change(rc);
        gm.set_msg_seq(self.seq);
        gm.set_sender_id(self.id);
        gm.set_receiver_id(id);
        self.seq += 1;
        return gm;
    }

    fn do_state(&mut self) {
        let now = Instant::now();
        if (now - self.game_instant).as_millis() > self.game.config.state_delay_ms() as u128 {
            let dead_ids = self.game.update();
            for dead_id in dead_ids {
                if let Some(player) = self.game.players.get(&dead_id) {
                    let mut gm = GameMessage::new();
                    let mut rc = RoleChangeMsg::new();
                    rc.set_sender_role(NodeRole::MASTER);
                    rc.set_receiver_role(NodeRole::VIEWER);
                    gm.set_role_change(rc);
                    gm.set_msg_seq(self.seq);
                    gm.set_sender_id(self.id);
                    gm.set_receiver_id(player.id());
                    self.seq += 1;
                    self.pending_msgs.insert(gm.msg_seq(), PendingMsg::new(&gm, player.ip_address().to_socket_addrs().unwrap().next().unwrap()));
                }
            }
            self.get_state();
            self.game_instant = now;
        }
    }

    fn check_timeout(&mut self) {
        let now = Instant::now();
        let delay = self.game.config.state_delay_ms() as f32 / 1000f32 * 0.8f32;
        self.player_timeout.retain(|addr, timeout| {
            if (now - *timeout).as_secs_f32() > delay {
                if let Some(id) = self.addrs.get(addr) {
                    self.game.players.remove(id);
                }
                self.addrs.remove(addr);
                return false;
            }
            true
        })
    }

    fn send(&mut self, msg: &GameMessage, addr: SocketAddr) {
        self.pending_msgs.insert(msg.msg_seq(), PendingMsg::new(&msg, addr));
    }

    fn ack_pending(&mut self, seq: i64) {
        let _res = self.pending_msgs.remove(&seq);
    }

    fn update_timeout(&mut self, addr: SocketAddr) {
        self.player_timeout.insert(addr, Instant::now());
    }

    fn check_pending(&mut self) {
        let now = Instant::now();
        let delay = self.game.config.state_delay_ms() as u128 / 10;
        self.pending_msgs.retain(|_, pending_msg| {
            if let Some(send_time) = pending_msg.send_time {
                if (now - send_time).as_millis() > delay {
                    pending_msg.send(&mut self.sockets).expect("send pending");
                    pending_msg.send_time = Some(now);
                }
            } else {
                pending_msg.send(&mut self.sockets).expect("send pending");
                pending_msg.send_time = Some(now);
            }
            return pending_msg.send_count < 5;
        })
    }
    
    pub fn get_state(&mut self) {
        let mut state = GameState::new();
        state.foods.clone_from(&self.game.food);
        state.snakes = self.game.snakes.values().cloned().collect();
        let mut players = GamePlayers::new();
        players.players = self.game.players.values().cloned().collect();
        state.players = Some(players).into();
        state.set_state_order(self.game.order);
        let mut state_msg = StateMsg::new();
        state_msg.state = Some(state).into();
        let mut msg = GameMessage::new();
        msg.set_state(state_msg);
        msg.set_sender_id(self.id);
        let pm = &mut self.pending_msgs;
        for player in self.game.players.values() {
            msg.set_msg_seq(self.seq);
            msg.set_receiver_id(player.id());
            pm.insert(msg.msg_seq(), PendingMsg::new(&msg, player.ip_address().to_socket_addrs().expect("player ip to socket").next().expect("one ip")));
            // Self::send(pm, &msg, player.ip_address().to_socket_addrs().expect("player ip to socket").next().expect("one ip"));
            // self.send(&msg, player.ip_address().to_socket_addrs().expect("player ip to socket").next().expect("one ip"));
            self.seq += 1;
        }
    }
    
    fn send_ack(&self, seq: i64, receiver_id: Option<i32>, addr: &SocketAddr) {
        let mut game_msg = GameMessage::new();
        game_msg.set_ack(AckMsg::new());
        game_msg.set_sender_id(self.id);
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
                        game_message::Type::Ping(_ping_msg) => {
                            // self.get_player_by_ip(&addr).and_then(|player| player.set_ping(ping_msg.ping()));
                            self.send_ack(msg.msg_seq(), Some(msg.sender_id()), &addr);
                            self.update_timeout(addr);
                        }
                        game_message::Type::Ack(_ack) => {
                            // self.send_ack(msg.msg_seq(), Some(msg.sender_id()), &addr)
                            self.ack_pending(msg.msg_seq());
                            self.update_timeout(addr);
                        }
                        game_message::Type::Announcement(_announcement) => {

                        }
                        game_message::Type::Discover(_discover) => {
                            let announcement = self.get_announcement();
                            if let Ok(bytes) = announcement.write_to_bytes() {
                                let _ = self.sockets.socket.send_to(&bytes, &addr);
                            }
                        }
                        game_message::Type::Error(_error) => {
                            self.send_ack(msg.msg_seq(), Some(msg.sender_id()), &addr);
                            self.update_timeout(addr);
                        }
                        game_message::Type::Join(join) => {
                            self.send_ack(msg.msg_seq(), Some(self.player_id), &addr);
                            let mut player = GamePlayer::new();
                            player.set_name(join.player_name().to_string());
                            player.set_ip_address(addr.to_string());
                            player.set_role(join.requested_role()); // TODO: change
                            player.set_type(join.player_type());
                            player.set_id(self.player_id);
                            self.player_id += 1;
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
                                    self.addrs.insert(addr, player.id());
                                    self.game.players.insert(player.id(), player);
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
                                self.addrs.insert(addr, player.id());
                                self.game.players.insert(player.id(), player);
                            }
                            self.update_timeout(addr);
                        }
                        game_message::Type::RoleChange(role_change) => {
                            self.send_ack(msg.msg_seq(), Some(msg.sender_id()), &addr);
                            if let Some(id) = self.addrs.get(&addr) {
                                let free_5x5 = self.game.get_free_coord5x5();
                                if let Some(player) = self.game.players.get_mut(id) {
                                    let new_role = role_change.sender_role();
                                    if new_role != player.role() {
                                        if new_role == NodeRole::VIEWER {
                                            self.game.snakes.entry(*id).and_modify(|snak| snak.set_state(SnakeState::ZOMBIE));
                                            player.set_role(new_role);
                                        } else if new_role == NodeRole::NORMAL {
                                            if let Some(snak) = self.game.snakes.get_mut(&player.id()) {
                                                snak.set_state(SnakeState::ALIVE);
                                                player.set_role(new_role);
                                            } else {
                                                if let Some((head, tail, dir)) = free_5x5 {
                                                    self.game.world[head.y() as usize][head.x() as usize] = WorldCell::Snake;
                                                    self.game.world[tail.y() as usize][tail.x() as usize] = WorldCell::Snake;
                                                    let mut snake = Snake::new();
                                                    snake.points.push(tail);
                                                    snake.points.push(head);
                                                    snake.set_head_direction(dir);
                                                    snake.set_state(SnakeState::ALIVE);
                                                    snake.set_player_id(*id);
                                                    self.game.snakes.insert(*id, snake);
                                                    player.set_role(new_role);
                                                } else {
                                                    
                                                }
                                            }
                                        }
                                    } 
                                }
                            }
                            self.update_timeout(addr);
                        }
                        game_message::Type::State(_state) => {
                            self.send_ack(msg.msg_seq(), Some(msg.sender_id()), &addr);
                            self.update_timeout(addr);
                        }
                        game_message::Type::Steer(steer) => {
                            self.send_ack(msg.msg_seq(), Some(msg.sender_id()), &addr);
                            if let Some(id) = self.addrs.get(&addr) {
                                if let Some(player) = self.game.players.get(id) {
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
                            }
                            self.update_timeout(addr);
                        }
                    }
                }
            }
        }
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
        self.addrs.insert( "127.0.0.1:48668".to_socket_addrs().unwrap().next().unwrap(), player.id());
        self.game.players.insert(player.id(), player);
        local_client
    }

    pub fn run(&mut self, player_name: &str) {
        let mut client = self.add_local_player(player_name);
        client.prepare();
        loop {
            self.receive_message();
            self.do_state();
            self.check_pending();
            self.check_timeout();
            self.do_multicast();
            client.action();
        }
    }
}

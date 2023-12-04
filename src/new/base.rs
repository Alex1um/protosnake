
pub mod game {

use std::{collections::HashMap, time::Duration, cell::Cell};

use rand::rngs::ThreadRng;
use rand::Rng;

pub struct Config {
    pub width: usize,
    pub height: usize,
    pub food_count: usize,
    pub state_delay: Duration
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Coord {
    pub x: i32,
    pub y: i32,
}

impl Coord {
    pub fn new(x: i32, y: i32) -> Self {
        Coord { x, y }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum WorldCell {
    Snake,
    Empty,
    Food,
}

#[derive(Clone, Copy)]
pub enum Direction {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

pub struct Snake {
    pub head_direction: Direction,
    pub body: Vec<Coord>,
    pub id: i32,
}


pub trait World {
    fn copy(&mut self, world: &Self);
    fn get_free_random_coord(&self, rng: &mut ThreadRng) -> Coord;
}

impl World for Vec<Vec<WorldCell>> {
    fn copy(&mut self, world: &Self) {
        for (i, row) in world.iter().enumerate() {
            for (j, cell) in row.iter().enumerate() {
                self[i][j] = *cell;
            }
        }
    }
    
    // fn get_free_coords_shuffled(&self) -> Vec<(usize, usize)> {
    //     let mut free_coords = Vec::<(usize, usize)>::new();

    //     for (i, row) in self.world.iter().enumerate() {
    //         for (j, cell) in row.iter().enumerate() {
    //             if *cell == Cell::None {
    //                 free_coords.push((j, i));
    //             }
    //         }
    //     }
    //     let mut rng = self.rng.take();
    //     free_coords.shuffle(&mut rng);
    //     self.rng.set(rng);
    //     free_coords
    // }

    fn get_free_random_coord(&self, rng: &mut ThreadRng) -> Coord {
        let coord = loop {
            let row = rng.gen_range(0..self.len());
            let col = rng.gen_range(0..self[0].len());

            if self[row][col] == WorldCell::Empty {
                let coord = Coord::new(col as i32, row as i32);
                break coord;
            }
        };
        coord
    }
}

pub struct Game {
    pub world: Vec<Vec<WorldCell>>,
    tmp_world: Vec<Vec<WorldCell>>,
    pub snakes: HashMap<i32, Snake>,
    pub food: Vec<Coord>,
    pub config: Config,
    rng: Cell<ThreadRng>,
}

impl Game {
    pub fn new(config: Config) -> Self {
        Game {
            world: vec![vec![WorldCell::Empty; config.width]; config.height],
            tmp_world: vec![vec![WorldCell::Empty; config.width]; config.height],
            snakes: HashMap::new(),
            food: Vec::with_capacity(config.food_count),
            config,
            rng: Cell::new(rand::thread_rng()),
        }
    }

    pub fn add_snake(&mut self, snake: Snake) {
        for e in snake.body.iter() {
            self.world[e.y][e.x] = 
        }
        self.snakes.insert(snake.id, snake);
    }

    pub fn tick(&mut self) {
        let snakes = &mut self.snakes;
        let world = &mut self.world;
        let new_world = &mut self.tmp_world;
        snakes.retain(|id, snake| {
            if let Some(head) = snake.body.last() {
                let mut new_x = head.x;
                let mut new_y = head.y;
                match snake.head_direction {
                    Direction::UP => new_y -= 1,
                    Direction::DOWN => new_y += 1,
                    Direction::LEFT => new_x -= 1,
                    Direction::RIGHT => new_x += 1
                }
                let size_y = self.config.height as i32;
                let size_x = self.config.width as i32;
                new_y = (new_y + size_y) % size_y;
                new_x = (new_x + size_x) % size_x;

                let new_coord = Coord::new(new_x, new_y);

                let new_y = new_y as usize;
                let new_x = new_x as usize;

                match world[new_y][new_x] {
                    WorldCell::Empty => {
                        snake.body.push(new_coord);
                        let tail = snake.body.remove(0);
                        new_world[tail.y as usize][tail.x as usize] = WorldCell::Empty;
                        new_world[new_y][new_x] = WorldCell::Snake;
                        return true;
                    }
                    WorldCell::Food => {
                        snake.body.push(new_coord);
                        new_world[new_y][new_x] = WorldCell::Snake;
                        self.food.remove(self.food.iter().position(|x| x == &new_coord).expect("food coord"));
                        return true;
                    }
                    WorldCell::Snake => {
                        let mut rng = self.rng.take();
                        for coord in snake.body.drain(..) {
                            if rng.gen_bool(0.5f64) {
                                // self.add_food_to(coord);
                                new_world[coord.y as usize][coord.x as usize] = WorldCell::Food;
                                self.food.push(coord);
                            } else {
                                // self.clear_coord(coord);
                                new_world[coord.y as usize][coord.x as usize] = WorldCell::Empty;
                            }
                        }
                        self.rng.set(rng);
                        return false;
                    }
                }
            } else { false }
        });

        let mut rng = self.rng.take();
        for _ in 0..(self.food.len() - self.config.food_count) {
            let coord = new_world.get_free_random_coord(&mut rng);
            self.food.push(coord);
            new_world[coord.y as usize][coord.y as usize] = WorldCell::Food;
        }
        self.rng.set(rng);

        world.copy(&new_world);
    }    
}

}

pub mod protobuf {

    use crate::snakes::snakes;
    use super::game;
    // use std::vec;

    impl<'a> From<&'a game::Coord> for snakes::game_state::Coord {
        fn from(value: &game::Coord) -> Self {
            let mut coord = snakes::game_state::Coord::new();
            coord.set_x(value.x);
            coord.set_y(value.y);
            coord
        }
    }

    impl<'a> FromIterator<&'a game::Coord> for Vec<snakes::game_state::Coord> {
        fn from_iter<T: IntoIterator<Item = &'a game::Coord>>(iter: T) -> Self {
            let mut v = Vec::new();
            for e in iter {
                v.push(e.into())
            }
            v
        }
    }

    impl<'a> From<&'a game::Snake> for snakes::game_state::Snake {
        fn from(value: &'a game::Snake) -> Self {
            let mut proto_snake = snakes::game_state::Snake::new();
            proto_snake.set_player_id(value.id);
            proto_snake.points = value.body.iter().collect();
            proto_snake
            
        }
    }

    impl<'a> FromIterator<&'a game::Snake> for Vec<snakes::game_state::Snake> {
        fn from_iter<T: IntoIterator<Item = &'a game::Snake>>(iter: T) -> Self {
            let mut v = Vec::new();
            for e in iter {
                v.push(e.into())
            }
            v
        }
    }

}

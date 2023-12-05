use protobuf::MessageField;
use rand::Rng;
use rand::rngs::ThreadRng;
use crate::snakes::snakes::{Direction, GamePlayer, GameConfig, GameState};
use crate::snakes::snakes::game_state::{Coord, Snake};
use std::cell::Cell;
use std::collections::HashMap;
use rand::prelude::SliceRandom;
use rand::seq::IteratorRandom;

#[derive(Copy, Clone, PartialOrd, PartialEq)]
pub enum WorldCell {
    Snake,
    Food,
    None,
}

pub struct Game {
    pub world: Vec<Vec<WorldCell>>,
    pub food: Vec<Coord>,
    pub snakes: HashMap<i32, Snake>,
    pub config: GameConfig,
    rng: Cell<ThreadRng>,
}

impl Game {

    pub fn get_free_coords_shuffled(&self) -> Vec<(usize, usize)> {
        let mut free_coords = Vec::<(usize, usize)>::new();

        for (i, row) in self.world.iter().enumerate() {
            for (j, cell) in row.iter().enumerate() {
                if *cell == WorldCell::None {
                    free_coords.push((j, i));
                }
            }
        }
        let mut rng = self.rng.take();
        free_coords.shuffle(&mut rng);
        self.rng.set(rng);
        free_coords
    }

    pub fn get_free_random_coord(&self) -> Coord {
        let mut rng = self.rng.take();

        let coord = loop {
            let row = rng.gen_range(0..self.world.len());
            let col = rng.gen_range(0..self.world[0].len());

            if self.world[row][col] == WorldCell::None {
                let mut coord = Coord::new();
                coord.set_x(col as i32);
                coord.set_y(row as i32);
                break coord;
            }
        };
        self.rng.set(rng);
        coord
    }
    
    pub fn get_free_coord5x5(&self) -> Option<(Coord, Coord, Direction)> {

        for y in 2..self.config.height() - 3 {
            'cel: for x in 2..self.config.width() - 3 {
                for i in (y - 2)..=(y + 2) {
                    for j in (x - 2)..=(x + 2) {
                        if self.world[i as usize][j as usize] == WorldCell::Snake {
                            continue 'cel;
                        }
                    }
                }
                let mut rng = self.rng.take();
                if let Some((tx, ty, direction)) = [
                    (x - 1, y, Direction::RIGHT),
                    (x + 1, y, Direction::LEFT),
                    (x, y - 1, Direction::DOWN),
                    (x, y + 1, Direction::UP)
                    ].into_iter()
                    .filter(|(x, y, _)| {
                        self.world[*y as usize][*x as usize] != WorldCell::Food
                    })
                    .choose(&mut rng) 
                    {
                    self.rng.set(rng);
                    let mut head_coord = Coord::new();
                    head_coord.set_x(x);
                    head_coord.set_x(y);
                    let mut tail_coord = Coord::new();
                    tail_coord.set_x(tx);
                    tail_coord.set_x(ty);
                    return Some((head_coord, tail_coord, direction));
                } else {
                    self.rng.set(rng);
                    continue 'cel;
                }
                    
            }
        }
        return None;
    }

    pub fn new(config: GameConfig) -> Self {
        Game {
            world: vec![vec![WorldCell::None; config.width() as usize]; config.height() as usize],
            food: vec![],
            rng: Cell::new(rand::thread_rng()),
            snakes: HashMap::new(),
            config,
        }
    }

    fn add_food_to(&mut self, coord: Coord) {
        self.world[coord.y() as usize][coord.x() as usize] = WorldCell::Food;
        self.food.push(coord);
    }

    fn clear_coord(&mut self, coord: Coord) {
        self.world[coord.y() as usize][coord.x() as usize] = WorldCell::None;
    }

    pub fn update(&mut self) {
        let snakes = &mut self.snakes;
        let mut new_world  = self.world.clone();
        snakes.retain(|pid, snake| {
            if let Some(head) = snake.points.last() {
                let mut new_x = head.x.unwrap();
                let mut new_y = head.y.unwrap();
                match snake.head_direction.unwrap().unwrap() {
                    Direction::UP => new_y -= 1,
                    Direction::DOWN => new_y += 1,
                    Direction::LEFT => new_x -= 1,
                    Direction::RIGHT => new_x += 1
                }
                let size_y = new_world.len() as i32;
                let size_x = new_world[0].len() as i32;
                new_y = (new_y + size_y) % size_y;
                new_x = (new_x + size_x) % size_x;

                let mut new_coord = Coord::new();
                new_coord.set_x(new_x);
                new_coord.set_y(new_y);

                let new_y = new_y as usize;
                let new_x = new_x as usize;

                match new_world[new_y][new_x] {
                    WorldCell::None => {
                        snake.points.push(new_coord);
                        let tail = snake.points.remove(0);
                        // self.clear_coord(tail);
                        new_world[tail.y() as usize][tail.x() as usize] = WorldCell::None;
                        new_world[new_y][new_x] = WorldCell::Snake;
                        return true;
                    }
                    WorldCell::Food => {
                        snake.points.push(new_coord);
                        new_world[new_y][new_x] = WorldCell::Snake;
                        return true;
                    }
                    WorldCell::Snake => {
                        let mut rng = self.rng.take();
                        for coord in snake.points.drain(..) {
                            if rng.gen_bool(0.5f64) {
                                // self.add_food_to(coord);
                                new_world[coord.y() as usize][coord.x() as usize] = WorldCell::Food;
                                self.food.push(coord);
                            } else {
                                // self.clear_coord(coord);
                                new_world[coord.y() as usize][coord.x() as usize] = WorldCell::None;
                            }
                        }
                        self.rng.set(rng);
                        return false;
                    }
                }
            } else { false }
        });

        for _ in 0..(self.config.food_static()- self.food.len() as i32) {
            let coord = self.get_free_random_coord();
            self.add_food_to(coord);
        }

        self.world = new_world;
    }

    pub fn apply_state(&mut self, state: MessageField<GameState>) {
        self.snakes.clear();
        for row in self.world.iter_mut() {
            row.fill(WorldCell::None);
        }
        for snake in &state.snakes {
            for coord in &snake.points {
                self.world[coord.y() as usize][coord.x() as usize] = WorldCell::Snake;
            }
            self.snakes.entry(snake.player_id()).or_insert(snake.clone());
        }
        for coord in &state.foods {
            self.world[coord.y() as usize][coord.x() as usize] = WorldCell::Food;
        }
        self.food = state.foods.clone();
    }

}

pub trait Player {
}


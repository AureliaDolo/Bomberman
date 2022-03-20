use std::{
    collections::HashSet,
    time::{Duration, Instant},
};

use ggez::{
    conf,
    event::{self, EventHandler, KeyCode},
    graphics::{self, Color, Rect},
    input::keyboard,
    timer, ContextBuilder, GameError, GameResult,
};
use glam::Vec2;
use itertools::Itertools;
use rand::{prelude::SliceRandom, random, thread_rng};

const DELTA: f32 = 0.005;
const BOMB_TIMEOUT: std::time::Duration = Duration::from_secs(3);
const EXPLOSION_TIMEOUT: std::time::Duration = Duration::from_secs(3);
const BREAKABLE_PROPORTION: f32 = 0.6;
const BONUS_PROPORTION: f32 = 0.1;

fn main() -> GameResult {
    let c = conf::Conf::new();
    let height = 11;
    let width = 11;
    let mut rng = thread_rng();
    let start_points = [
        (1, 1),
        (1, height - 2),
        (width - 2, 1),
        (width - 2, height - 2),
    ];
    let walls = (0..11)
        .flat_map(|i| [(0, i), (i, 0), (i, height - 1), (width - 1, i)])
        .chain((0..width / 2).flat_map(|x| (0..height / 2).map(move |y| (2 * x, 2 * y))))
        .collect();

    let world = World::new(height, width, walls, &start_points);
    let x_grid_to_window = c.window_mode.width / world.width as f32;
    let y_grid_to_window = c.window_mode.height / world.height as f32;
    let player_size = x_grid_to_window.min(y_grid_to_window) * 0.8;

    let mut choosen = start_points.choose_multiple(&mut rng, 2);
    // Not supposed to fail.
    let start1 = choosen.next().unwrap_or(&start_points[0]);
    let player1 = Player {
        pos_x: start1.0 as f32 + 0.5,
        pos_y: start1.1 as f32 + 0.5,
        up: KeyCode::Z,
        down: KeyCode::S,
        right: KeyCode::D,
        left: KeyCode::Q,
        bomb: KeyCode::W,
        bomb_range: 1,
        kill_score: 0,
        alive: true,
    };
    let start2 = choosen.next().unwrap_or(&start_points[0]);
    let player2 = Player {
        pos_x: start2.0 as f32 + 0.5,
        pos_y: start2.1 as f32 + 0.5,
        up: KeyCode::Up,
        down: KeyCode::Down,
        right: KeyCode::Right,
        left: KeyCode::Left,
        bomb: KeyCode::Space,
        bomb_range: 1,
        kill_score: 0,
        alive: true,
    };

    let state = State {
        x_grid_to_window,
        y_grid_to_window,
        world,
        player_size,
        players: [player1, player2],
    };

    let (ctx, event_loop) = ContextBuilder::new("bomberman", "Aurélia")
        .default_conf(c)
        .build()
        .unwrap();

    event::run(ctx, event_loop, state);
}

struct State {
    players: [Player; 2],
    world: World,
    x_grid_to_window: f32,
    y_grid_to_window: f32,
    player_size: f32,
}

#[derive(Debug)]
struct Player {
    pos_x: f32,
    pos_y: f32,
    up: KeyCode,
    down: KeyCode,
    right: KeyCode,
    left: KeyCode,
    bomb: KeyCode,
    bomb_range: usize,
    kill_score: u8,
    alive: bool,
}

struct World {
    height: usize,
    width: usize,
    walls: Vec<Vec<Content>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Content {
    Nothing,
    Wall,
    Breakable,
    Bomb(Instant, usize),      // player_id
    Explosion(Instant, usize), // player_id
    StartPoint,
    Bonus(Bonus),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Bonus {}

impl World {
    fn new(
        height: usize,
        width: usize,
        walls_set: HashSet<(usize, usize)>,
        start_points: &[(usize, usize)],
    ) -> World {
        let mut walls = vec![vec![Content::Nothing; width]; height];
        for (x, y) in walls_set {
            walls[x][y] = Content::Wall
        }
        for (x, y) in start_points {
            walls[*y][*x] = Content::StartPoint
        }

        for (x, y) in (0..width).cartesian_product(0..height) {
            if matches!(walls[y][x], Content::Nothing) && random::<f32>() < BREAKABLE_PROPORTION {
                walls[y][x] = Content::Breakable
            }
        }
        World {
            height,
            width,
            walls,
        }
    }
}
impl State {
    fn check_collision_up_right(&self, x: f32, y: f32) -> bool {
        let up_right = (
            ((x * self.x_grid_to_window + self.player_size / 2.) / self.x_grid_to_window) as usize,
            ((y * self.y_grid_to_window - self.player_size / 2.) / self.y_grid_to_window) as usize,
        );

        matches!(
            self.world.walls[up_right.1][up_right.0],
            Content::Wall | Content::Breakable //| Content::Bomb(_)
        )
    }

    fn check_collision_up_left(&self, x: f32, y: f32) -> bool {
        let up_left = (
            ((x * self.x_grid_to_window - self.player_size / 2.) / self.x_grid_to_window) as usize,
            ((y * self.y_grid_to_window - self.player_size / 2.) / self.y_grid_to_window) as usize,
        );
        matches!(
            self.world.walls[up_left.1][up_left.0],
            Content::Wall | Content::Breakable //| Content::Bomb(_)
        )
    }

    fn check_collision_down_right(&self, x: f32, y: f32) -> bool {
        let down_right = (
            ((x * self.x_grid_to_window + self.player_size / 2.) / self.x_grid_to_window) as usize,
            ((y * self.y_grid_to_window + self.player_size / 2.) / self.y_grid_to_window) as usize,
        );
        matches!(
            self.world.walls[down_right.1][down_right.0],
            Content::Wall | Content::Breakable // | Content::Bomb(_)
        )
    }
    fn check_collision_down_left(&self, x: f32, y: f32) -> bool {
        let down_left = (
            ((x * self.x_grid_to_window - self.player_size / 2.) / self.x_grid_to_window) as usize,
            ((y * self.y_grid_to_window + self.player_size / 2.) / self.y_grid_to_window) as usize,
        );
        matches!(
            self.world.walls[down_left.1][down_left.0],
            Content::Wall | Content::Breakable //| Content::Bomb(_)
        )
    }

    fn check_collision_up(&self, x: f32, y: f32) -> bool {
        self.check_collision_up_left(x, y) || self.check_collision_up_right(x, y)
    }

    fn check_collision_down(&self, x: f32, y: f32) -> bool {
        self.check_collision_down_left(x, y) || self.check_collision_down_right(x, y)
    }

    fn check_collision_left(&self, x: f32, y: f32) -> bool {
        self.check_collision_up_left(x, y) || self.check_collision_down_left(x, y)
    }

    fn check_collision_right(&self, x: f32, y: f32) -> bool {
        self.check_collision_down_right(x, y) || self.check_collision_up_right(x, y)
    }

    fn player_update(
        &mut self,
        player_id: usize,
        ctx: &mut ggez::Context,
    ) -> Result<(), GameError> {
        // bomb management
        {
            let mut player = &mut self.players[player_id];
            if !player.alive {
                return Ok(());
            }

            if keyboard::is_key_pressed(ctx, player.bomb) {
                self.world.walls[player.pos_y as usize][player.pos_x as usize] =
                    Content::Bomb(Instant::now(), player_id)
            }

            match self.world.walls[player.pos_y as usize][player.pos_x as usize] {
                Content::Explosion(_, killer_id) => {
                    println!("Player {} was killed by player {}!", player_id, killer_id);
                    player.alive = false;
                    self.players[killer_id].kill_score += 1;
                    return Ok(());
                }
                Content::Bonus(_) => {
                    println!("Here's a bonus ! Player {} bomb range increased", player_id);
                    player.bomb_range += 1;
                    self.world.walls[player.pos_y as usize][player.pos_x as usize] =
                        Content::Nothing
                }
                _ => {}
            }
        }

        {
            let player = &self.players[player_id];
            // update position
            let (mut new_x, mut new_y) = (player.pos_x, player.pos_y);
            if keyboard::is_key_pressed(ctx, player.right)
                && !self.check_collision_right(new_x + DELTA, new_y)
            {
                new_x += DELTA
            }
            if keyboard::is_key_pressed(ctx, player.left)
                && !self.check_collision_left(new_x - DELTA, new_y)
            {
                new_x -= DELTA
            }
            if keyboard::is_key_pressed(ctx, player.down)
                && !self.check_collision_down(new_x, new_y + DELTA)
            {
                new_y += DELTA
            }
            if keyboard::is_key_pressed(ctx, player.up)
                && !self.check_collision_up(new_x, new_y - DELTA)
            {
                new_y -= DELTA
            }
            let mut player = &mut self.players[player_id];
            player.pos_y = new_y;
            player.pos_x = new_x;
        }
        Ok(())
    }

    fn propagate_explosion(
        &mut self,
        x: usize,
        y: usize,
        delta_x: isize,
        delta_y: isize,
        range: usize,
        player_id: usize,
    ) -> GameResult {
        for i in 1..=range {
            let new_x = (x as isize + delta_x * i as isize) as usize;
            let new_y = (y as isize + delta_y * i as isize) as usize;
            if matches!(
                // Ugly and assumes a at least size one border until the end of the world
                self.world.walls[new_y][new_x],
                Content::Wall
            ) {
                break;
            } else if matches!(self.world.walls[new_y][new_x], Content::Breakable) {
                self.world.walls[new_y][new_x] = Content::Explosion(Instant::now(), player_id);
                break;
            } else {
                self.world.walls[new_y][new_x] = Content::Explosion(Instant::now(), player_id);
            }
        }
        Ok(())
    }
}

impl EventHandler<GameError> for State {
    fn update(&mut self, ctx: &mut ggez::Context) -> Result<(), GameError> {
        if self.players.iter().all(|p| !p.alive) {
            // all players are dead
            ggez::event::quit(ctx)
        }
        const DESIRED_FPS: u32 = 60;
        while timer::check_update_time(ctx, DESIRED_FPS) {
            for i in 0..self.players.len() {
                self.player_update(i, ctx)?;
            }

            // explosions
            for (x, y) in (0..self.world.width).cartesian_product(0..self.world.height) {
                match self.world.walls[y][x] {
                    Content::Bomb(i, killer_id) => {
                        if i.elapsed() >= BOMB_TIMEOUT {
                            let range = self
                                .players
                                .get(killer_id)
                                .expect("Player not found")
                                .bomb_range;

                            self.world.walls[y][x] = Content::Explosion(Instant::now(), killer_id);
                            self.propagate_explosion(x, y, -1, 0, range, killer_id)?; // up
                            self.propagate_explosion(x, y, 1, 0, range, killer_id)?; // down
                            self.propagate_explosion(x, y, 0, 1, range, killer_id)?; // right
                            self.propagate_explosion(x, y, 0, -1, range, killer_id)?;
                            // left
                        }
                    }
                    Content::Explosion(i, _) => {
                        if i.elapsed() >= EXPLOSION_TIMEOUT {
                            self.world.walls[y][x] = if random::<f32>() <= BONUS_PROPORTION {
                                Content::Bonus(Bonus {})
                            } else {
                                Content::Nothing
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> Result<(), GameError> {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        let player = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            Rect {
                x: -self.player_size / 2.,
                y: -self.player_size / 2.,
                w: self.player_size,
                h: self.player_size,
            },
            Color::WHITE,
        )?;
        let bomb = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            Vec2::new(self.x_grid_to_window / 2., self.y_grid_to_window / 2.),
            self.player_size / 2.,
            0.01,
            Color::BLACK,
        )?;

        let explosion = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            Vec2::new(self.x_grid_to_window / 2., self.y_grid_to_window / 2.),
            self.player_size / 2.,
            0.01,
            Color::RED,
        )?;

        let brick = graphics::Mesh::new_rounded_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            Rect {
                x: 0.,
                y: 0.,
                w: self.x_grid_to_window,
                h: self.y_grid_to_window,
            },
            0.1,
            Color::BLACK,
        )?;
        let breakable = graphics::Mesh::new_rounded_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            Rect {
                x: self.x_grid_to_window * 0.05,
                y: self.y_grid_to_window * 0.05,
                w: self.x_grid_to_window * 0.9,
                h: self.y_grid_to_window * 0.9,
            },
            1.,
            Color::new(0.5, 0.5, 0.5, 1.),
        )?;

        let bonus = graphics::Mesh::new_rounded_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            Rect {
                x: self.x_grid_to_window * 0.15,
                y: self.y_grid_to_window * 0.15,
                w: self.x_grid_to_window * 0.7,
                h: self.y_grid_to_window * 0.7,
            },
            1.,
            Color::new(1., 0.5, 0.6, 1.),
        )?;

        for (x, y) in (0..self.world.width).cartesian_product(0..self.world.height) {
            match self.world.walls[y][x] {
                Content::Wall => {
                    graphics::draw(
                        ctx,
                        &brick,
                        (Vec2::new(
                            x as f32 * self.x_grid_to_window,
                            y as f32 * self.y_grid_to_window,
                        ),),
                    )?;
                }
                Content::Explosion(..) => {
                    graphics::draw(
                        ctx,
                        &explosion,
                        (Vec2::new(
                            x as f32 * self.x_grid_to_window,
                            y as f32 * self.y_grid_to_window,
                        ),),
                    )?;
                }
                Content::Bomb(..) => {
                    graphics::draw(
                        ctx,
                        &bomb,
                        (Vec2::new(
                            x as f32 * self.x_grid_to_window,
                            y as f32 * self.y_grid_to_window,
                        ),),
                    )?;
                }
                Content::Breakable => {
                    graphics::draw(
                        ctx,
                        &breakable,
                        (Vec2::new(
                            x as f32 * self.x_grid_to_window,
                            y as f32 * self.y_grid_to_window,
                        ),),
                    )?;
                }
                Content::Bonus(_) => {
                    graphics::draw(
                        ctx,
                        &bonus,
                        (Vec2::new(
                            x as f32 * self.x_grid_to_window,
                            y as f32 * self.y_grid_to_window,
                        ),),
                    )?;
                }
                _ => {}
            }
        }
        for p in self.players.iter() {
            if p.alive {
                graphics::draw(
                    ctx,
                    &player,
                    (Vec2::new(
                        p.pos_x * self.x_grid_to_window,
                        p.pos_y * self.y_grid_to_window,
                    ),),
                )?;
            }
        }

        graphics::present(ctx)?;
        Ok(())
    }
}

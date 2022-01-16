use std::collections::HashSet;

use ggez::{
    conf,
    event::{self, EventHandler, KeyCode},
    graphics::{self, Color, Rect},
    input::keyboard,
    ContextBuilder, GameError, GameResult,
};
use glam::Vec2;
use itertools::Itertools;

const DELTA: f32 = 0.002;

fn main() -> GameResult {
    let c = conf::Conf::new();
    let height = 11;
    let width = 11;
    let walls = (0..11)
        .map(|i| vec![(0, i), (i, 0), (i, height - 1), (width - 1, i)])
        .flatten()
        .chain(
            (0..width / 2)
                .map(|x| (0..height / 2).map(move |y| (2 * x, 2 * y)))
                .flatten(),
        )
        .collect();
    //let walls = HashSet::new();

    let world = World::new(height, width, walls);
    let x_grid_to_window = c.window_mode.width / world.width as f32;
    let y_grid_to_window = c.window_mode.height / world.height as f32;
    let state = State {
        pos_x: width as f32 / 2.,
        pos_y: height as f32 / 2.,
        x_grid_to_window,
        y_grid_to_window,
        world,
        player_size: if x_grid_to_window < y_grid_to_window {
            x_grid_to_window * 0.8
        } else {
            y_grid_to_window * 0.8
        },
    };
    let (ctx, event_loop) = ContextBuilder::new("bomberman", "Aurélia")
        .default_conf(c)
        .build()
        .unwrap();

    event::run(ctx, event_loop, state);
}

struct State {
    pos_x: f32,
    pos_y: f32,
    world: World,
    x_grid_to_window: f32,
    y_grid_to_window: f32,
    player_size: f32,
}

struct World {
    height: usize,
    width: usize,
    walls: Vec<Vec<Content>>,
}

#[derive(Debug, Clone, Copy)]
enum Content {
    Nothing,
    Wall,
    Breakable,
    Bomb,
    StartPoint,
    Bonus(Bonus),
}

#[derive(Debug, Clone, Copy)]
struct Bonus {}

impl World {
    fn new(height: usize, width: usize, walls_set: HashSet<(usize, usize)>) -> World {
        let mut walls = vec![vec![Content::Nothing; width]; height];
        for (x, y) in walls_set {
            walls[x][y] = Content::Wall
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
            Content::Wall | Content::Breakable | Content::Bomb
        )
    }

    fn check_collision_up_left(&self, x: f32, y: f32) -> bool {
        let up_left = (
            ((x * self.x_grid_to_window - self.player_size / 2.) / self.x_grid_to_window) as usize,
            ((y * self.y_grid_to_window - self.player_size / 2.) / self.y_grid_to_window) as usize,
        );
        matches!(
            self.world.walls[up_left.1][up_left.0],
            Content::Wall | Content::Breakable | Content::Bomb
        )
    }

    fn check_collision_down_right(&self, x: f32, y: f32) -> bool {
        let down_right = (
            ((x * self.x_grid_to_window + self.player_size / 2.) / self.x_grid_to_window) as usize,
            ((y * self.y_grid_to_window + self.player_size / 2.) / self.y_grid_to_window) as usize,
        );
        matches!(
            self.world.walls[down_right.1][down_right.0],
            Content::Wall | Content::Breakable | Content::Bomb
        )
    }
    fn check_collision_down_left(&self, x: f32, y: f32) -> bool {
        let down_left = (
            ((x * self.x_grid_to_window - self.player_size / 2.) / self.x_grid_to_window) as usize,
            ((y * self.y_grid_to_window + self.player_size / 2.) / self.y_grid_to_window) as usize,
        );
        matches!(
            self.world.walls[down_left.1][down_left.0],
            Content::Wall | Content::Breakable | Content::Bomb
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
}

impl EventHandler<GameError> for State {
    fn update(&mut self, ctx: &mut ggez::Context) -> Result<(), GameError> {
        let (mut new_x, mut new_y) = (self.pos_x, self.pos_y);
        if keyboard::is_key_pressed(ctx, KeyCode::Right)
            && !self.check_collision_right(new_x + DELTA, new_y)
        {
            new_x += DELTA
        }
        if keyboard::is_key_pressed(ctx, KeyCode::Left)
            && !self.check_collision_left(new_x - DELTA, new_y)
        {
            new_x -= DELTA
        }
        if keyboard::is_key_pressed(ctx, KeyCode::Down)
            && !self.check_collision_down(new_x, new_y + DELTA)
        {
            new_y += DELTA
        }
        if keyboard::is_key_pressed(ctx, KeyCode::Up)
            && !self.check_collision_up(new_x, new_y - DELTA)
        {
            new_y -= DELTA
        }

        self.pos_x = new_x;
        self.pos_y = new_y;

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
            Color::RED,
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
                _ => {}
            }
        }
        graphics::draw(
            ctx,
            &player,
            (Vec2::new(
                self.pos_x * self.x_grid_to_window,
                self.pos_y * self.y_grid_to_window,
            ),),
        )?;

        graphics::present(ctx)?;
        Ok(())
    }
}

use ggez::{
    conf,
    event::{self, EventHandler, KeyCode},
    graphics::{self, Color, Rect},
    input::keyboard,
    ContextBuilder, GameError, GameResult,
};
use glam::Vec2;

const DELTA: f32 = 0.2;

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

    let world = World {
        height,
        width,
        walls,
    };
    let state = State {
        pos_x: c.window_mode.width / 2.,
        pos_y: c.window_mode.height / 2.,
        x_grid_to_window: c.window_mode.width / world.width as f32,
        y_grid_to_window: c.window_mode.height / world.height as f32,
        world,
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
}

struct World {
    height: u64,
    width: u64,
    walls: Vec<(u64, u64)>,
}

impl EventHandler<GameError> for State {
    fn update(&mut self, ctx: &mut ggez::Context) -> Result<(), GameError> {
        if keyboard::is_key_pressed(ctx, KeyCode::Right) {
            self.pos_x += DELTA;
        } else if keyboard::is_key_pressed(ctx, KeyCode::Left) {
            self.pos_x -= DELTA;
        } else if keyboard::is_key_pressed(ctx, KeyCode::Down) {
            self.pos_y += DELTA;
        } else if keyboard::is_key_pressed(ctx, KeyCode::Up) {
            self.pos_y -= DELTA;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> Result<(), GameError> {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        let circle = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            Vec2::new(0.0, 0.0),
            10.0,
            0.5,
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
            Color::GREEN,
        )?;
        for wall in &self.world.walls {
            graphics::draw(
                ctx,
                &brick,
                (Vec2::new(
                    wall.0 as f32 * self.x_grid_to_window,
                    wall.1 as f32 * self.y_grid_to_window,
                ),),
            )?;
        }
        graphics::draw(ctx, &circle, (Vec2::new(self.pos_x, self.pos_y),))?;

        graphics::present(ctx)?;
        Ok(())
    }
}

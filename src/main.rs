use ggez::{
    conf::{self, WindowMode},
    event::{self, EventHandler, KeyCode, KeyMods},
    graphics::{self, Color},
    input::keyboard,
    ContextBuilder, GameError, GameResult,
};
use glam::Vec2;

const DELTA: f32 = 0.2;

fn main() -> GameResult {
    let c = conf::Conf::new();
    let state = State {
        pos_x: c.window_mode.width / 2.,
        pos_y: c.window_mode.height / 2.,
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
        graphics::draw(ctx, &circle, (Vec2::new(self.pos_x, self.pos_y),))?;

        graphics::present(ctx)?;
        Ok(())
    }
}

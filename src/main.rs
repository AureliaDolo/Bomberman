use std::time::Duration;

use ggez::{
    conf,
    event::{self, EventHandler},
    timer, ContextBuilder, GameError,
};

fn main() {
    let state = State {
        dt: Duration::new(0, 0),
    };

    let c = conf::Conf::new();
    let (ctx, event_loop) = ContextBuilder::new("bomberman", "Aurélia")
        .default_conf(c)
        .build()
        .unwrap();

    event::run(ctx, event_loop, state);
}

struct State {
    dt: Duration,
}

impl EventHandler<GameError> for State {
    fn update(&mut self, ctx: &mut ggez::Context) -> Result<(), GameError> {
        self.dt = timer::delta(ctx);
        Ok(())
    }

    fn draw(&mut self, _ctx: &mut ggez::Context) -> Result<(), GameError> {
        println!("Hello ggez! dt = {}ns", self.dt.as_nanos());
        Ok(())
    }
}

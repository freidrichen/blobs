use ggez::event::{self, EventHandler};
use ggez::graphics;
use ggez::{Context, ContextBuilder, GameResult};
use mint::Point2;

struct GameState {}

impl GameState {
    fn new(ctx: &Context) -> GameState {
        GameState {}
    }
}

impl EventHandler for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::WHITE);
        let blob = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            Point2 { x: 100.0, y: 100.0 },
            40.0,
            0.5,
            (128, 128, 128).into(),
        )?;
        graphics::draw(ctx, &blob, graphics::DrawParam::new())?;
        graphics::present(ctx)
    }
}

fn main() {
    // Make a Context and an EventLoop.
    let (mut ctx, mut event_loop) = ContextBuilder::new("Blobs", "Freidrichen").build().unwrap();

    // Create an instance of your event handler.
    // Usually, you should provide it with the Context object
    // so it can load resources like images during setup.
    let mut my_game = GameState::new(&mut ctx);

    match event::run(&mut ctx, &mut event_loop, &mut my_game) {
        Ok(_) => println!("Exited cleanly."),
        Err(e) => println!("Error occured: {}", e),
    }
}

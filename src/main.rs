use ggez::event::{self, EventHandler};
use ggez::graphics;
use ggez::{Context, ContextBuilder, GameResult};
use nalgebra::{Point2, Vector2};

struct GameState {
    aim_vec: Vector2<f32>,
    center: Point2<f32>,
}

impl GameState {
    fn new(ctx: &Context) -> GameState {
        GameState {
            aim_vec: Vector2::x(),
            center: Point2::new(100.0, 100.0),
        }
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
            self.center,
            40.0,
            0.5,
            (128, 128, 128).into(),
        )?;
        graphics::draw(ctx, &blob, graphics::DrawParam::new())?;
        let aim = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            self.center + 50.0*self.aim_vec,
            4.0,
            1.0,
            (200, 200, 200).into(),
        )?;
        graphics::draw(ctx, &aim, graphics::DrawParam::new())?;
        graphics::present(ctx)
    }

    fn mouse_motion_event(&mut self, ctx: &mut Context, x: f32, y: f32, _dx: f32, _dy: f32) {
        let cursor_pos = Point2::new(x, y);
        self.aim_vec = (cursor_pos - self.center).normalize();
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

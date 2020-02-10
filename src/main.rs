use ggez::event::{self, EventHandler};
use ggez::graphics;
use ggez::{Context, ContextBuilder, GameResult};
use nalgebra::{Point2, Vector2};

// SPRING_CONSTANT is physical spring constant divided by blob mass
const SPRING_CONST: f32 = 20.0;
const SPRING_EQ_LEN: f32 = 40.0;
const DAMPING_CONST: f32 = 0.01;
const G: f32 = 10.0;

struct GameState {
    center: Point2<f32>,
    vel: Vector2<f32>,
    aim_vec: Vector2<f32>,
}

impl GameState {
    fn new(_ctx: &Context) -> GameState {
        GameState {
            center: Point2::new(100.0, 100.0),
            vel: Vector2::zeros(),
            aim_vec: Vector2::x(),
        }
    }
}

impl EventHandler for GameState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        let dt = 0.1;
        let spring_vec = Point2::new(400.0, 0.0) - self.center;
        let spring_vec = if spring_vec.norm() < SPRING_EQ_LEN {
            0.0
        } else {
            (spring_vec.norm() - SPRING_EQ_LEN) / spring_vec.norm() / spring_vec.norm()
        } * spring_vec;
        let acc_spring = SPRING_CONST * spring_vec;
        let acc_damping = -DAMPING_CONST * self.vel;
        let acc_gravity = G * Vector2::y();
        let acc_tot = acc_spring + acc_gravity + acc_damping;

        self.vel += acc_tot * dt;
        self.center += self.vel * dt;

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
            self.center + 50.0 * self.aim_vec,
            4.0,
            1.0,
            (200, 200, 200).into(),
        )?;
        graphics::draw(ctx, &aim, graphics::DrawParam::new())?;
        let line = graphics::Mesh::new_line(
            ctx,
            &[self.center, Point2::new(400.0, 0.0)],
            4.0,
            (200, 200, 200).into(),
        )?;
        graphics::draw(ctx, &line, graphics::DrawParam::new())?;
        graphics::present(ctx)
    }

    fn mouse_motion_event(&mut self, _ctx: &mut Context, x: f32, y: f32, _dx: f32, _dy: f32) {
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

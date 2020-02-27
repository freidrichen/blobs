use ggez::event::{self, EventHandler};
use ggez::graphics;
use ggez::{Context, GameResult};
use ggez::input::mouse::MouseButton;
use nalgebra::{Point2, Vector2};

// SPRING_CONSTANT is physical spring constant divided by blob mass
const SPRING_CONST: f32 = 20.0;
const SPRING_EQ_LEN: f32 = 40.0;
const DAMPING_CONST: f32 = 0.01;
const G: f32 = 10.0;
const HOOK_TRAVELING_SPEED: f32 = 150.0;

const BLOB_RADIUS: f32 = 40.0;
const SCREEN_SIZE: (f32, f32) = (1000.0, 1000.0);

enum HookState {
    Hooked(Point2<f32>),
    Traveling(Point2<f32>, Vector2<f32>),
    None,
}

struct GameState {
    center: Point2<f32>,
    vel: Vector2<f32>,
    aim_vec: Vector2<f32>,
    hook: HookState,
}

impl GameState {
    fn new(_ctx: &Context) -> GameState {
        GameState {
            center: Point2::new(100.0, 100.0),
            vel: Vector2::zeros(),
            aim_vec: Vector2::x(),
            hook: HookState::Hooked(Point2::new(400.0, 0.0)),
        }
    }
}

impl EventHandler for GameState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        let dt = 0.1;
        let acc_spring = if let HookState::Hooked(hook_point) = self.hook {
            let spring_vec = hook_point - self.center;
            (if spring_vec.norm() < SPRING_EQ_LEN {
                0.0
            } else {
                (spring_vec.norm() - SPRING_EQ_LEN) / spring_vec.norm() / spring_vec.norm()
            }) * SPRING_CONST * spring_vec
        } else {
            Vector2::zeros()
        };
        let acc_damping = -DAMPING_CONST * self.vel;
        let acc_gravity = G * Vector2::y();
        let acc_tot = acc_spring + acc_gravity + acc_damping;

        // Update blob position and velocity
        self.vel += acc_tot * dt;
        self.center += self.vel * dt;
        if let Some((alpha, collision_normal)) = wall_blob_collision(self.center, self.vel) {
            // Mirror velocity in the plane defined by normal vector.
            self.vel -= 2.0*self.vel.dot(&collision_normal)*collision_normal;
            self.center -= 2.0*(self.center - ()).dot(&collision_normal)*collision_normal;
        }

        // Update hook position
        if let HookState::Traveling(hook_point, hook_vel) = self.hook {
            let hook_point = hook_point + hook_vel * dt;
            self.hook = match wall_point_collision(hook_point) {
                Some(collision_point) => HookState::Hooked(collision_point),
                None => HookState::Traveling(hook_point, hook_vel),
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::WHITE);
        let blob = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            self.center,
            BLOB_RADIUS,
            0.5,
            (128, 128, 128).into(),
        )?;
        graphics::draw(ctx, &blob, graphics::DrawParam::new())?;
        let aim = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            self.center + (BLOB_RADIUS + 10.0) * self.aim_vec,
            4.0,
            1.0,
            (200, 200, 200).into(),
        )?;
        graphics::draw(ctx, &aim, graphics::DrawParam::new())?;
        if let HookState::Hooked(hook_point) | HookState::Traveling(hook_point, _) = self.hook {
            let hook = graphics::Mesh::new_line(
                ctx,
                &[self.center, hook_point],
                4.0,
                (200, 200, 200).into(),
            )?;
            graphics::draw(ctx, &hook, graphics::DrawParam::new())?;
        }
        graphics::present(ctx)
    }

    fn mouse_motion_event(&mut self, _ctx: &mut Context, x: f32, y: f32, _dx: f32, _dy: f32) {
        let cursor_pos = Point2::new(x, y);
        self.aim_vec = (cursor_pos - self.center).normalize();
        // TODO: Ensure that aim_vec can never be (0, 0)
    }

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, button: MouseButton, _x: f32, _y: f32) {
        if button == MouseButton::Right {
            self.hook = HookState::None;
        } else if button == MouseButton::Left {
            self.hook = HookState::Traveling(
                self.center + self.aim_vec,
                HOOK_TRAVELING_SPEED * self.aim_vec
            )
        }
    }
}

/// Look for collision between blob and walls.
/// Returns the point of collision and the normal vector,
/// or None if no collision has occurred.
fn wall_blob_collision(blob_center: Point2<f32>, last_step_vec: Vector2<f32>) -> Option<(f32, Vector2<f32>)> {
    let x = blob_center.coords.x;
    let y = blob_center.coords.y;
    if x < BLOB_RADIUS {
        let alpha = (BLOB_RADIUS - x)/last_step_vec.x;
        Some((alpha, Vector2::x()))
    } else if x > SCREEN_SIZE.0 - BLOB_RADIUS {
        let alpha = (BLOB_RADIUS - x)/last_step_vec.x;
        Some((Point2::new(SCREEN_SIZE.0, y), -Vector2::x()))
    } else if y < BLOB_RADIUS {
        let alpha = (BLOB_RADIUS - y)/last_step_vec.y; // How far into wall
        Some((Point2::new(x, 0.0), Vector2::y()))
    } else if y > SCREEN_SIZE.1 - BLOB_RADIUS {
        let alpha = (BLOB_RADIUS - x)/last_step_vec.y;
        Some((Point2::new(x, SCREEN_SIZE.1), -Vector2::y()))
    } else {
        None
    }
}

/// Look for collision between point p and walls.
/// Returns the point of collision if any, otherwise returns None.
fn wall_point_collision(p: Point2<f32>) -> Option<Point2<f32>> {
    let x = p.coords.x;
    let y = p.coords.y;
    if x < 0.0 {
        Some(Point2::new(0.0, y))
    } else if x > SCREEN_SIZE.0 {
        Some(Point2::new(SCREEN_SIZE.0, y))
    } else if y < 0.0 {
        Some(Point2::new(x, 0.0))
    } else if y > SCREEN_SIZE.1 {
        Some(Point2::new(x, SCREEN_SIZE.1))
    } else {
        None
    }
}

fn main() {
    let (mut ctx, mut event_loop) = ggez::ContextBuilder::new("Blobs", "Freidrichen")
        .window_setup(ggez::conf::WindowSetup::default().title("Blobs!"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(SCREEN_SIZE.0, SCREEN_SIZE.1))
        .build().unwrap();
    let mut my_game = GameState::new(&mut ctx);

    match event::run(&mut ctx, &mut event_loop, &mut my_game) {
        Ok(_) => println!("Exited cleanly."),
        Err(e) => println!("Error occured: {}", e),
    }
}

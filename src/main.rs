use ggez::event::{self, EventHandler};
use ggez::graphics;
use ggez::input::mouse::{self, MouseButton};
use ggez::{Context, GameResult};
use nalgebra::{Point2, Vector2};
use std::collections::HashMap;

// SPRING_CONSTANT is physical spring constant divided by blob mass
const SPRING_CONST: f32 = 20.0;
const SPRING_EQ_LEN: f32 = 40.0;
const DAMPING_CONST: f32 = 0.01;
const G: f32 = 10.0;
const HOOK_TRAVELING_SPEED: f32 = 150.0;

const BLOB_RADIUS: f32 = 40.0;
const SCREEN_SIZE: (f32, f32) = (1000.0, 1000.0);

const LOCAL_ID: usize = 0;

enum HookState {
    Hooked(Point2<f32>),
    Traveling(Point2<f32>, Vector2<f32>),
    None,
}

struct Blob {
    center: Point2<f32>,
    vel: Vector2<f32>,
    aim_vec: Vector2<f32>,
    hook: HookState,
}

impl Blob {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
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
        if let Some((_collision_point, collision_normal)) = wall_blob_collision(self.center) {
            // Mirror velocity in the plane defined by normal vector.
            self.vel -= 2.0 * self.vel.dot(&collision_normal) * collision_normal;

            // TODO: Move center out of wall too. This is important for when the
            // next turns forces (e.g. gravity) are strong so the flipped
            // velocity is not enough to escape the wall. Try moving close to
            // the ground with low vertical velocity to see an example of this.
        }
        // TODO: Ensure that aim_vec can never be (0, 0)
        let mouse_pos: Point2<f32> = mouse::position(ctx).into();
        self.aim_vec = (mouse_pos - self.center).normalize();

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
        Ok(())
    }
}

struct GameState {
    blobs: HashMap<usize, Blob>,
}

impl GameState {
    fn new(_ctx: &Context) -> GameState {
        let mut blobs = HashMap::new();
        blobs.insert(
            0,
            Blob {
                center: Point2::new(100.0, 100.0),
                vel: Vector2::zeros(),
                aim_vec: Vector2::x(),
                hook: HookState::Hooked(Point2::new(400.0, 0.0)),
            },
        );
        blobs.insert(
            10,
            Blob {
                center: Point2::new(200.0, 100.0),
                vel: Vector2::new(10.0, 10.0),
                aim_vec: Vector2::x(),
                hook: HookState::Hooked(Point2::new(0.0, 0.0)),
            },
        );
        GameState { blobs }
    }
}

impl EventHandler for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        for (_id, blob) in self.blobs.iter_mut() {
            blob.update(ctx)?
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::WHITE);
        for (_id, blob) in self.blobs.iter_mut() {
            blob.draw(ctx)?;
        }
        graphics::present(ctx)
    }

    fn mouse_motion_event(&mut self, _ctx: &mut Context, x: f32, y: f32, _dx: f32, _dy: f32) {
        let cursor_pos = Point2::new(x, y);
        // TODO: Ensure that aim_vec can never be (0, 0)
        self.blobs
            .entry(LOCAL_ID)
            .and_modify(|blob| blob.aim_vec = (cursor_pos - blob.center).normalize());
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        _x: f32,
        _y: f32,
    ) {
        if button == MouseButton::Right {
            self.blobs
                .entry(LOCAL_ID)
                .and_modify(|blob| blob.hook = HookState::None);
        } else if button == MouseButton::Left {
            self.blobs.entry(LOCAL_ID).and_modify(|blob| {
                blob.hook = HookState::Traveling(
                    blob.center + blob.aim_vec,
                    HOOK_TRAVELING_SPEED * blob.aim_vec,
                )
            });
        }
    }
}

/// Look for collision between blob and walls.
/// Returns the point of collision and the normal vector,
/// or None if no collision has occurred.
fn wall_blob_collision(blob_center: Point2<f32>) -> Option<(Point2<f32>, Vector2<f32>)> {
    let x = blob_center.coords.x;
    let y = blob_center.coords.y;
    if x < BLOB_RADIUS {
        Some((Point2::new(0.0, y), Vector2::x()))
    } else if x > SCREEN_SIZE.0 - BLOB_RADIUS {
        Some((Point2::new(SCREEN_SIZE.0, y), -Vector2::x()))
    } else if y < BLOB_RADIUS {
        Some((Point2::new(x, 0.0), Vector2::y()))
    } else if y > SCREEN_SIZE.1 - BLOB_RADIUS {
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
        .window_setup(
            ggez::conf::WindowSetup::default()
                .title("Blobs!")
                .vsync(true),
        )
        .window_mode(ggez::conf::WindowMode::default().dimensions(SCREEN_SIZE.0, SCREEN_SIZE.1))
        .build()
        .unwrap();
    let mut my_game = GameState::new(&mut ctx);

    match event::run(&mut ctx, &mut event_loop, &mut my_game) {
        Ok(_) => println!("Exited cleanly."),
        Err(e) => println!("Error occured: {}", e),
    }
}

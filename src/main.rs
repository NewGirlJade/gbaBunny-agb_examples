#![no_std]
#![no_main]
// This is required to allow writing tests
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]

// By default no_std crates don't get alloc, so you won't be able to use things like Vec
// until you declare the extern crate. `agb` provides an allocator so it will all work
extern crate alloc;

use agb::display::GraphicsFrame;
use agb::display::object::Object;
use agb::fixnum::{Rect, Vector2D, rect, vec2};
use agb::include_aseprite;
include_aseprite!(
    mod sprites,
    "gfx/sprites.aseprite"
);

pub struct Paddle {
    pos: Vector2D<i32>,
    facing: Orientation,
}
#[derive(PartialEq, Eq)]
pub enum Orientation {
    L,
    R,
}
impl Paddle {
    pub fn new(start_x: i32, start_y: i32, start_facing: Orientation) -> Self {
        Self {
            pos: vec2(start_x, start_y),
            facing: start_facing,
        }
    }
    pub fn set_pos(&mut self, x: i32, y: i32) {
        self.pos = vec2(x, y);
    }
    pub fn move_by(&mut self, y: i32) {
        self.pos.y = (self.pos.y + y).clamp(0, agb::display::HEIGHT - (16 * 3));
    }
    pub fn show(&self, frame: &mut GraphicsFrame) {
        let mut top = Object::new(sprites::PADDLE_END.sprite(0));
        top.set_pos(self.pos);
        if self.facing == Orientation::R {
            top.set_hflip(true);
        }
        top.show(frame);
        let mut middle = Object::new(sprites::PADDLE_MID.sprite(0));
        middle.set_pos(self.pos + vec2(0, 16));
        if self.facing == Orientation::R {
            middle.set_hflip(true);
        }
        middle.show(frame);
        let mut bottom = Object::new(sprites::PADDLE_END.sprite(0));
        bottom.set_pos(self.pos + vec2(0, 32));
        bottom.set_vflip(true);
        if self.facing == Orientation::R {
            bottom.set_hflip(true);
        }
        bottom.show(frame);
    }
    pub fn collision_rect(&self) -> Rect<i32> {
        rect(self.pos + vec2(2, 6), vec2(12, 16 * 3 - 6))
    }
}

// The main function must take 1 arguments and never returns, and must be marked with
// the #[agb::entry] macro.
#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let mut gfx = gba.graphics.get();
    let mut button_controller = agb::input::ButtonController::new();

    let mut ball = Object::new(sprites::BALL.sprite(0));
    let mut ball_position = vec2(50, 50);
    let mut ball_velocity = vec2(-2, -1);

    let mut paddle_a = Paddle::new(8, 8, Orientation::L);
    let mut paddle_b = Paddle::new(240 - 16 - 8, 8, Orientation::R);

    loop {
        button_controller.update();
        let mut paddle_a_move = button_controller.y_tri() as i32;
        if button_controller.is_pressed(agb::input::Button::A) {
            paddle_a_move *= 2;
        }
        paddle_a.move_by(paddle_a_move);

        if ball_position.y < paddle_b.pos.y + 32 {
            paddle_b.move_by(-1);
        }
        if ball_position.y > paddle_b.pos.y + 16 {
            paddle_b.move_by(1);
        }

        let potential_ball_position = ball_position + ball_velocity;
        let ball_rect = rect(potential_ball_position + vec2(2, 2), vec2(12, 12));
        if paddle_a.collision_rect().touches(ball_rect) {
            ball_velocity.x = 2;
        }
        if paddle_b.collision_rect().touches(ball_rect) {
            ball_velocity.x = -2;
        }
        if potential_ball_position.x <= 0 || potential_ball_position.x >= agb::display::WIDTH - 16 {
            ball_velocity.x *= -1
        }
        if potential_ball_position.y <= 0 || potential_ball_position.y >= agb::display::HEIGHT - 16
        {
            ball_velocity.y *= -1
        }

        ball_position += ball_velocity;
        ball.set_pos(ball_position);

        let mut frame = gfx.frame();

        ball.show(&mut frame);
        paddle_a.show(&mut frame);
        paddle_b.show(&mut frame);

        frame.commit();
    }
}

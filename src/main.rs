#![no_std]
#![no_main]
// This is required to allow writing tests
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]

// By default no_std crates don't get alloc, so you won't be able to use things like Vec
// until you declare the extern crate. `agb` provides an allocator so it will all work
extern crate alloc;

use agb::include_aseprite;
include_aseprite!(
    mod sprites,
    "gfx/sprites.aseprite"
);

use agb::display::object::Object;

// The main function must take 1 arguments and never returns, and must be marked with
// the #[agb::entry] macro.
#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let mut gfx = gba.graphics.get();
    let mut ball = Object::new(sprites::BALL.sprite(0));
    let mut frame = gfx.frame();
    ball.show(&mut frame);
    frame.commit();

    let mut ball_x = 50;
    let mut ball_y = 50;
    let mut ball_x_vel = 1;
    let mut ball_y_vel = 1;
    loop {
        ball_x = (ball_x + ball_x_vel).clamp(0, agb::display::WIDTH - 16);

        ball_y = (ball_y + ball_y_vel).clamp(0, agb::display::HEIGHT - 16);
        if ball_x == 0 || ball_x == agb::display::WIDTH - 16 {
            ball_x_vel = -ball_x_vel;
        }

        if ball_y == 0 || ball_y == agb::display::HEIGHT - 16 {
            ball_y_vel = -ball_y_vel;
        }
        ball.set_pos((ball_x, ball_y));

        let mut frame = gfx.frame();
        ball.show(&mut frame);

        frame.commit();
    }
}

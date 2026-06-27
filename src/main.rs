#![no_std]
#![no_main]
// This is required to allow writing tests
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]

// By default no_std crates don't get alloc, so you won't be able to use things like Vec
// until you declare the extern crate. `agb` provides an allocator so it will all work
extern crate alloc;

use agb::display::object::Object;
use agb::display::tiled::{RegularBackground, RegularBackgroundSize, TileFormat};
use agb::display::{Graphics, GraphicsFrame, Priority};
use agb::fixnum::{Num, Rect, Vector2D, num, rect, vec2};
use agb::{Gba, include_aseprite};
include_aseprite!(
    mod sprites,
    "gfx/sprites.aseprite"
);
use agb::include_background_gfx;
use agb::input::ButtonController;
include_background_gfx!(
    mod background,
    PLAY_FIELD => deduplicate "gfx/background.aseprite",
);

type Fixed = Num<i32, 8>;

pub struct Paddle {
    position: Vector2D<Fixed>,
    facing: Orientation,
}
#[derive(PartialEq, Eq)]
pub enum Orientation {
    L,
    R,
}
impl Paddle {
    pub fn new(start_x: Fixed, start_y: Fixed, start_facing: Orientation) -> Self {
        Self {
            position: vec2(start_x, start_y),
            facing: start_facing,
        }
    }
    pub fn set_pos(&mut self, x: Fixed, y: Fixed) {
        self.position = vec2(x, y);
    }
    pub fn move_by(&mut self, y: Fixed) {
        self.position.y =
            (self.position.y + y).clamp(num!(0), num!(agb::display::HEIGHT - (16 * 3)));
    }
    pub fn show(&self, frame: &mut GraphicsFrame) {
        let sprite_position = self.position.round();

        let mut top = Object::new(sprites::PADDLE_END.sprite(0));
        top.set_pos(sprite_position);
        if self.facing == Orientation::R {
            top.set_hflip(true);
        }
        top.show(frame);
        let mut middle = Object::new(sprites::PADDLE_MID.sprite(0));
        middle.set_pos(sprite_position + vec2(0, 16));
        if self.facing == Orientation::R {
            middle.set_hflip(true);
        }
        middle.show(frame);
        let mut bottom = Object::new(sprites::PADDLE_END.sprite(0));
        bottom.set_pos(sprite_position + vec2(0, 32));
        bottom.set_vflip(true);
        if self.facing == Orientation::R {
            bottom.set_hflip(true);
        }
        bottom.show(frame);
    }
    pub fn collision_rect(&self) -> Rect<Fixed> {
        rect(
            self.position + (vec2(num!(4), num!(6))),
            vec2(num!(8), num!(16 * 3 - 6)),
        )
    }
}

pub struct Ball {
    position: Vector2D<Fixed>,
    velocity: Vector2D<Fixed>,
}
impl Ball {
    pub fn new(position: Vector2D<Fixed>, velocity: Vector2D<Fixed>) -> Self {
        Self { position, velocity }
    }

    pub fn update(&mut self, paddle_a: &Paddle, paddle_b: &Paddle) {
        let potential_ball_position = self.position + self.velocity;

        let ball_rect = rect(
            potential_ball_position + vec2(num!(2), num!(2)),
            vec2(num!(12), num!(12)),
        );
        if paddle_a.collision_rect().touches(ball_rect) {
            self.velocity.x = num!(2);
            let y_diff = (ball_rect.centre().y - paddle_a.collision_rect().centre().y) / 32;
            self.velocity.y += y_diff;
        }
        if paddle_b.collision_rect().touches(ball_rect) {
            self.velocity.x = num!(-2);
            let y_diff = (ball_rect.centre().y - paddle_b.collision_rect().centre().y) / 32;
            self.velocity.y += y_diff;
        }
        if potential_ball_position.x <= num!(0)
            || potential_ball_position.x >= num!(agb::display::WIDTH - 16)
        {
            self.velocity.x *= -1
        }
        if potential_ball_position.y <= num!(0)
            || potential_ball_position.y >= num!(agb::display::HEIGHT - 16)
        {
            self.velocity.y *= -1
        }

        self.position += self.velocity;
    }
    pub fn show(&self, frame: &mut GraphicsFrame) {
        let sprite_position = self.position.round();

        Object::new(sprites::BALL.sprite(0))
            .set_pos(sprite_position)
            .show(frame);
    }
}

struct ComputerPaddle(Paddle);

impl ComputerPaddle {
    fn new() -> Self {
        Self(Paddle::new(num!(240 - 16 - 8), num!(8), Orientation::R))
    }

    fn update(self: &mut Self, ball: &Ball) {
        if ball.position.y < self.0.position.y + 28 {
            self.0.move_by(num!(-2)); //this number is what makes the AI easier or harder to bamboozle
        }
        if ball.position.y > self.0.position.y + 20 {
            self.0.move_by(num!(2));
        }
    }

    fn show(self: &mut Self, frame: &mut GraphicsFrame) {
        self.0.show(frame);
    }
}

struct HumanPaddle(Paddle);

impl HumanPaddle {
    fn new() -> Self {
         Self(Paddle::new(num!(8), num!(8), Orientation::L))
    }

    fn update(self: &mut Self, button_controller: &ButtonController) {
        let mut paddle_a_move = button_controller.y_tri() as i32;
        if button_controller.is_pressed(agb::input::Button::A) {
            paddle_a_move *= 2;
        }
        self.0.move_by(Fixed::from(paddle_a_move) * num!(1.2));
    }

    fn show(self: &mut Self, frame: &mut GraphicsFrame) {
        self.0.show(frame);
    }
}

struct Model {
    gba: Gba,
    button_controller: ButtonController,
    ball: Ball,
    human_paddle: HumanPaddle,
    computer_paddle: ComputerPaddle,
    bg: RegularBackground,
}

impl Model {
    fn new(mut gba: agb::Gba) -> Model {
        let mut gfx = gba.graphics.get();
        gfx.set_background_palettes(background::PALETTES);
        let mut bg = RegularBackground::new(
            Priority::P3,
            RegularBackgroundSize::Background32x32,
            TileFormat::FourBpp, //to do eight bpp you need to include 256 after the fat arrow in the include_background_gfx macro above to tell the program you want 256 color mode.
        );
        bg.fill_with(&background::PLAY_FIELD);
        let button_controller = agb::input::ButtonController::new();
        let ball = Ball::new(vec2(num!(51), num!(50)), vec2(num!(-2), num!(-0.5)));

        Model {
            gba,
            button_controller,
            ball,
            human_paddle: HumanPaddle::new(),
            computer_paddle: ComputerPaddle::new(),
            bg,
        }
    }

    fn update(self: &mut Self) {
        self.button_controller.update();

        // Move human paddle based on controller input
        self.human_paddle.update(&self.button_controller);

        // Move computer paddle based on where the ball is
        self.computer_paddle.update(&self.ball);

        // Move the ball, changing direction when it hits a wall or paddle
        self.ball.update(&self.human_paddle.0, &self.computer_paddle.0);
    }

    fn show(&mut self) {
        let mut gfx = self.gba.graphics.get();
        let mut frame = gfx.frame();

        self.ball.show(&mut frame);
        self.human_paddle.show(&mut frame);
        self.computer_paddle.show(&mut frame);
        self.bg.show(&mut frame);
        frame.commit();
    }
}

// The main function must take 1 arguments and never returns, and must be marked with
// the #[agb::entry] macro.
#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let mut model = Model::new(gba);
    loop {
        model.update();
        model.show();
    }
}

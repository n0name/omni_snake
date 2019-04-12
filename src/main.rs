extern crate math2d;
extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;

use math2d::math::common::*;
use math2d::math::vector::Vec2D;
use math2d::math::angle::Angle;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{ GlGraphics, OpenGL };
use std::f64::consts::PI;
use std::collections::VecDeque;

const SEGMENT_RADIUS: f64 = 5.0;    // [Pix]
const UPDATE_TIME: f64 = 0.1;       // [S]

struct Snake {
    dir: Vec2D,
    segments: VecDeque<Vec2D>,
}

impl Snake {
    fn new(start_pos: Vec2D, start_dir: Vec2D) -> Snake {
        let seg_separation = SEGMENT_RADIUS;
        let mut segments = VecDeque::new();
        segments.push_back(start_pos);
        segments.push_back(start_pos - start_dir * seg_separation);
        segments.push_back(start_pos - start_dir * seg_separation * 2.0);
        Snake {
            dir: start_dir,
            segments: segments
        }
    }
}

pub struct App {
    gl: GlGraphics, // OpenGL drawing backend.
    snake: Snake,
    last_move: f64
}

impl App {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
        const RED:   [f32; 4] = [1.0, 0.0, 0.0, 1.0];
        const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

        let segment_graphics = ellipse::Ellipse::new(GREEN);
        let snake = &self.snake;

        self.gl.draw(args.viewport(), |c, gl| {
            clear(BLACK, gl);
            snake.segments.iter().for_each(|s|{
                let (x, y) = s.as_tuple();
                let transform = c.transform.trans(x - 250.0, y - 250.0);
                segment_graphics.draw(ellipse::circle(x, y, SEGMENT_RADIUS), &c.draw_state, transform, gl);
            });
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        self.last_move -= args.dt;
        if self.last_move <= 0.0 {
            self.last_move = UPDATE_TIME;
            let head = self.snake.segments.front().unwrap() ;
            let seg = *head + self.snake.dir * SEGMENT_RADIUS;
            self.snake.segments.pop_back();
            self.snake.segments.push_front(seg);
        }
    }
}

fn main() {
    let opengl = OpenGL::V4_5;

    // Create an Glutin window.
    let mut window: Window = WindowSettings::new(
            "Omni Snake",
            [500, 500]
        )
        .opengl(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    // Create a new game and run it.
    let mut app = App {
        gl: GlGraphics::new(opengl),
        snake: Snake::new(Vec2D::new(250.0, 250.0), Vec2D::new(-1.0, 0.0)),
        last_move: 0.0
    };

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(r) = e.render_args() {
            app.render(&r);
        }

        if let Some(Button::Keyboard(btn)) = e.press_args() {
            match btn {
                Key::Left => app.snake.dir.rotate(&Angle::from_deg(-10.0)),
                Key::Right => app.snake.dir.rotate(&Angle::from_deg(10.0)),
                _ => ()
            }
        }

        if let Some(u) = e.update_args() {
            app.update(&u);
        }
    }
}
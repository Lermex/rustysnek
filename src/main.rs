use std::collections::LinkedList;
use rand::distributions::{IndependentSample, Range};

use piston_window::*;
use piston_window::Event::*;
use piston_window::Input::*;
use piston_window::Button::*;


extern crate piston_window;
extern crate rand;

const CELL_LENGTH: f64 = 21.0;

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;

type Point = (f64, f64);

fn main() {
    let orange = [1.0, 0.6, 0.0, 1.0];
    let red = [1.0, 0.0, 0.0, 1.0];

    let mut snek_body: LinkedList<Point> = LinkedList::new();

    snek_body.push_front((10.0, 10.0));
    snek_body.push_front((10.0, 31.0));
    snek_body.push_front((10.0, 52.0));

    let mut apple = (220.0, 220.0);

    let mut direction = Direction::Right;

    let mut window: piston_window::PistonWindow =
        piston_window::WindowSettings::new("Snek", [SCREEN_WIDTH, SCREEN_HEIGHT])
        .exit_on_esc(true).build().unwrap();

    let mut mov = 0.0;

    while let Some(e) = window.next() {
        direction = match e {
            Input(Press(Keyboard(Key::Left)))  => Direction::Left,
            Input(Press(Keyboard(Key::Right))) => Direction::Right,
            Input(Press(Keyboard(Key::Up)))    => Direction::Up,
            Input(Press(Keyboard(Key::Down)))  => Direction::Down,
            _ => direction,
        };
        if let Some(r) = e.update_args() {
            mov += r.dt;
        }
        if mov >= 1.0/15.0 {
            mov = mov - 1.0/15.0;
            update(&direction, &mut snek_body, &mut apple);
        }
        window.draw_2d(&e, |c, g| {
            piston_window::clear([0.0; 4], g);

            for segment in &snek_body {
                piston_window::rectangle(orange, [segment.0, segment.1, 20.0, 20.0], c.transform, g);
                piston_window::ellipse(red, [apple.0 + 5.0, apple.1 + 5.0, 10.0, 10.0], c.transform, g);
            }
        });
    }
}

fn update(direction: &Direction, snek_body: &mut LinkedList<Point>, apple: &mut Point) {
    let mut rng = rand::thread_rng();
    let width_range: Range<f64> = Range::new(5.0, (SCREEN_WIDTH - 10) as f64);
    let height_range: Range<f64> = Range::new(5.0, (SCREEN_HEIGHT - 10) as f64);

    let will_eat;
    let new_head = {
        let head = snek_body.front().unwrap();
        let delta = shift(direction);
        will_eat = should_eat(&head, &apple);
        (head.0 + delta.0, head.1 + delta.1)
    };

    if !will_eat {
        snek_body.pop_back();
    } else {
        let new_apple_x = (width_range.ind_sample(&mut rng) / CELL_LENGTH).round() * CELL_LENGTH + 10.0;
        let new_apple_y = (height_range.ind_sample(&mut rng) / CELL_LENGTH).round() * CELL_LENGTH + 10.0;
        *apple = (new_apple_x, new_apple_y);
    }

    snek_body.push_front(new_head);
}

fn shift(direction: &Direction) -> Point {
    match *direction {
        Direction::Up => (0.0, -21.0),
        Direction::Down => (0.0, 21.0),
        Direction::Left => (-21.0, 0.0),
        Direction::Right => (21.0, 0.0),
    }
}

fn should_eat(head: &Point, apple: &Point) -> bool {
    return head.0 <= apple.0 && apple.0 < head.0 + CELL_LENGTH
        && head.1 <= apple.1 && apple.1 < head.1 + CELL_LENGTH
}

enum Direction {
    Up, Down, Left, Right
}

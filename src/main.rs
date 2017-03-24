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
    let orange = |x: f32| [1.0, 0.6, 0.0, x];
    let blue   = |x: f32| [0.2, 0.2, 0.8, x];
    let red    = |x: f32| [1.0, 0.0, 0.0, x];

    let mut snek_body = LinkedList::<Point>::new();
    let mut walls = LinkedList::<Point>::new();

    snek_body.push_front((10.0, 10.0));
    snek_body.push_front((10.0, 31.0));
    snek_body.push_front((10.0, 52.0));

    walls.push_front((10.0, 10.0 + CELL_LENGTH * 10.0));

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
            update(&direction, &mut snek_body, &mut apple, &mut walls);
        }
        window.draw_2d(&e, |c, g| {
            piston_window::clear([0.0; 4], g);

            let len = (&snek_body).len() as f32;
            for (i, segment) in (&snek_body).iter().enumerate() {
                piston_window::rectangle(
                    orange(1.0 - (i as f32) / len * 0.9),
                    [segment.0, segment.1, 20.0, 20.0],
                    c.transform,
                    g);
            }
            for wall in &walls {
                piston_window::rectangle(
                    blue(1.0),
                    [wall.0, wall.1, 20.0, 20.0],
                    c.transform,
                    g);
            }
            piston_window::ellipse(
                red(1.0),
                [apple.0 + 5.0, apple.1 + 5.0, 10.0, 10.0],
                c.transform,
                g);
        });
    }
}

fn update(direction: &Direction, snek_body: &mut LinkedList<Point>, apple: &mut Point, walls: &mut LinkedList<Point>) {
    let mut rng = rand::thread_rng();
    let width_range: Range<f64> = Range::new(5.0, (SCREEN_WIDTH - 10) as f64);
    let height_range: Range<f64> = Range::new(5.0, (SCREEN_HEIGHT - 10) as f64);

    let will_eat_apple;
    let mut eat_tail: Option<usize> = None;
    let new_head = {
        let head = snek_body.front().unwrap();
        let delta = shift(direction);
        will_eat_apple = should_eat(&head, &apple);
        for (i, segment) in (&snek_body).iter().enumerate() {
            if i != 0 && should_eat(&head, &segment) {
                eat_tail = Some(i);
                break;
            }
        }
        (head.0 + delta.0, head.1 + delta.1)
    };

    if let Some(i) = eat_tail {
        let mut eaten_tail = snek_body.split_off(i);
        walls.append(&mut eaten_tail);
        snek_body.pop_back();
    } else if !will_eat_apple {
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

fn should_eat(head: &Point, thing: &Point) -> bool {
    return head.0 <= thing.0 && thing.0 < head.0 + CELL_LENGTH
        && head.1 <= thing.1 && thing.1 < head.1 + CELL_LENGTH
}

enum Direction {
    Up, Down, Left, Right
}

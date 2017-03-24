use std::collections::LinkedList;
use piston_window::*;
use piston_window::Event::*;
use piston_window::Input::*;
use piston_window::Button::*;

extern crate piston_window;

fn main() {
    let orange = [1.0, 0.6, 0.0, 1.0];

    let mut snek_body: LinkedList<(f64, f64)> = LinkedList::new();

    snek_body.push_front((10.0, 10.0));
    snek_body.push_front((10.0, 31.0));
    snek_body.push_front((10.0, 52.0));

    let mut x = 0.0;
    let mut direction = Direction::Right;

    let mut window: piston_window::PistonWindow = piston_window::WindowSettings::new("Snek", [800, 600])
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
        if mov >= 1.0 {
            mov = mov-1.0;
            update(&direction, &mut snek_body);
        }
        window.draw_2d(&e, |c, g| {
            piston_window::clear([0.0; 4], g);

            for segment in &snek_body {
                piston_window::rectangle(orange, [segment.0, segment.1, 20.0, 20.0], c.transform, g);
            }
        });

        x += 1.0;
    }
}

fn update(direction: &Direction, snek_body: &mut LinkedList<(f64, f64)>) {
    snek_body.pop_back();
    let new_head = {
        let head = snek_body.front().unwrap();
        let delta = shift(direction);
        (head.0 + delta.0, head.1 + delta.1)
    };
    snek_body.push_front(new_head);
}

fn shift(direction: &Direction) -> (f64, f64) {
    match *direction {
        Direction::Up => (0.0, -21.0),
        Direction::Down => (0.0, 21.0),
        Direction::Left => (-21.0, 0.0),
        Direction::Right => (21.0, 0.0),
    }
}

enum Direction {
    Up, Down, Left, Right
}

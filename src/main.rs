use std::collections::LinkedList;

extern crate piston_window;

fn main() {
    let orange = [1.0, 0.6, 0.0, 1.0];

    let mut snek_body: LinkedList<(f64, f64)> = LinkedList::new();

    snek_body.push_front((10.0, 10.0));
    snek_body.push_front((10.0, 31.0));
    snek_body.push_front((10.0, 52.0));

    let mut x = 0.0;
    let direction = Direction::Right;

    let mut window: piston_window::PistonWindow = piston_window::WindowSettings::new("Snek", [800, 600])
        .exit_on_esc(true).build().unwrap();

    while let Some(e) = window.next() {
        update(&mut snek_body);

        window.draw_2d(&e, |c, g| {
            piston_window::clear([0.0; 4], g);

            for segment in &snek_body {
                piston_window::rectangle(orange, [segment.0, segment.1, 20.0, 20.0], c.transform, g);
            }
        });

        x += 1.0;
    }
}

fn update(snek_body: &mut LinkedList<(f64, f64)>) {
    snek_body.pop_back();
    let new_head = {
        let head = snek_body.front().unwrap();
        (head.0 + 21.0, head.1)
    };
    snek_body.push_front(new_head);
}

enum Direction {
    Up, Down, Left, Right
}

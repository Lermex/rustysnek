use std::collections::LinkedList;
use rand::distributions::{IndependentSample, Range};
use rand::Rng;

use std::error::Error;

use std::net::UdpSocket;

use piston_window::*;
use piston_window::Event::*;
use piston_window::Input::*;
use piston_window::Button::*;
use uuid::Uuid;

extern crate piston_window;
extern crate rand;
extern crate uuid;

#[macro_use]
extern crate serde_derive;

extern crate serde_json;

const CELL_LENGTH: f64 = 21.0;

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;

type Point = (f64, f64);

fn main() {
    let orange = |x: f32| [1.0, 0.6, 0.0, x];
    let blue   = |x: f32| [0.2, 0.2, 0.8, x];
    let red    = |x: f32| [1.0, 0.0, 0.0, x];
    let green  = |x: f32| [0.0, 1.0, 0.0, x];

    let mut snek_body = LinkedList::<Point>::new();
    let mut walls = LinkedList::<Point>::new();
    let mut other_head = (0.0, 0.0);

    snek_body.push_front((10.0, 10.0));
    snek_body.push_front((10.0, 31.0));
    snek_body.push_front((10.0, 52.0));

    walls.push_front((10.0, 10.0 + CELL_LENGTH * 10.0));

    let mut apple = (220.0, 220.0);

    let mut direction = Direction::Right;

    let my_uuid = Uuid::new_v4();

    let mut window: piston_window::PistonWindow =
        piston_window::WindowSettings::new("Snek", [SCREEN_WIDTH, SCREEN_HEIGHT])
        .exit_on_esc(true).build().unwrap();

    let mut mov = 0.0;

    while let Some(e) = window.next() {

        match send_greetings(&my_uuid, &snek_body.front().unwrap()) {
            Ok(p) => {
                if p.0 != 0.0 {
                    other_head = p
                }
            },
            Err(x) => println!("send_greetings failed: {}", x),
        }

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

            piston_window::rectangle(
                green(1.0),
                [other_head.0, other_head.1, 20.0, 20.0],
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

fn send_greetings(my_uuid: &Uuid, own_head: &Point) -> Result<Point, Box<Error>> {
    let mut rng = rand::thread_rng();
    let id: i64 = rng.gen();
    println!("My id is: {}. Sending it to everybody", id);
    let message_text = format!("ID: {}", id);
    let socket = try!(UdpSocket::bind("0.0.0.0:34254"));
    socket.set_broadcast(true);
    println!("bound");
    let message = Message {id: my_uuid.clone(), point: own_head.clone()};
    let mut serialized = serde_json::to_string(&message).unwrap();
    println!("Serialized: {}", serialized);
    serialized.push('\n');
    try!(socket.send_to(&serialized.into_bytes()[..], ("255.255.255.255", 34254)));
    println!("sent");

    let data = read_buf(&socket)?;
    let deserialized: Message = serde_json::from_str(&data).unwrap();

    if deserialized.id != *my_uuid {
        println!("Got this point: {:?} from ID: {}", deserialized.point, deserialized.id);
        return Ok(deserialized.point);
    }


    return Ok((0.0, 0.0));
/*

    // send a reply to the socket we received data from
    let buf = &mut buf[..amt];
    buf.reverse();
    try!(socket.send_to(buf, &src));

*/

}

/// read from the socket until newline
fn read_buf(socket: &UdpSocket) -> Result<String, Box<Error>> {
    let mut data = String::new();
    loop {
        let mut buf = [0; 0x1000];
        try!(socket.recv_from(&mut buf));
        // todo: horribly inefficient, probably
        for &x in buf.iter() {
            if x == '\n' as u8 {
                return Ok(data);
            } else {
                data.push(x as char);
            }
        }
    }
}

enum Direction {
    Up, Down, Left, Right
}

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    id: Uuid,
    point: Point
}

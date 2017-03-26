use std::collections::LinkedList;
use rand::distributions::{IndependentSample, Range};
use rand::Rng;
use std::time::Duration;
use std::error::Error;
use std::net::UdpSocket;
use std::fmt;
use time::now;
use std::thread;
use std::sync::{RwLock, Arc};
use std::thread::sleep;

use piston_window::*;
use piston_window::Event::*;
use piston_window::Input::*;
use piston_window::Button::*;
use uuid::Uuid;

extern crate piston_window;
extern crate rand;
extern crate uuid;
extern crate time;

#[macro_use]
extern crate serde_derive;

extern crate serde_json;

const CELL_LENGTH: f64 = 21.0;

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;

type Point = (f64, f64);

struct State {
    snek_body: LinkedList<Point>,
    walls: LinkedList<Point>,
    other_body: LinkedList<Point>,
    apple: (f64, f64),
    direction: Direction
}

fn main() {
    let orange = |x: f32| [1.0, 0.6, 0.0, x];
    let blue   = |x: f32| [0.2, 0.2, 0.8, x];
    let red    = |x: f32| [1.0, 0.0, 0.0, x];
    let green  = |x: f32| [0.0, 1.0, 0.0, x];

    let mut init_state = State {
        snek_body: LinkedList::<Point>::new(),
        walls: LinkedList::<Point>::new(),
        other_body: LinkedList::<Point>::new(),
        apple: (220.0, 220.0),
        direction: Direction::Right
    };
    init_state.other_body.push_front((0.0, 0.0));

    init_state.snek_body.push_front((10.0, 10.0));
    init_state.snek_body.push_front((10.0, 31.0));
    init_state.snek_body.push_front((10.0, 52.0));

    init_state.walls.push_front((10.0, 10.0 + CELL_LENGTH * 10.0));

    let my_uuid = Uuid::new_v4();

    let mut window: piston_window::PistonWindow =
        piston_window::WindowSettings::new("Snek", [SCREEN_WIDTH, SCREEN_HEIGHT])
        .exit_on_esc(true).build().unwrap();

    let mut mov = 0.0;

    let lock = Arc::new(RwLock::new(init_state));
    {
        let lock = lock.clone();
        thread::spawn(move || {
            loop {
                let state = lock.read().unwrap();
                println!("got read lock in send loop");
                send(&my_uuid, &state);
                sleep(Duration::from_millis(30));
            }
        });
    }
    {
        let lock = lock.clone();
        thread::spawn(move || {
            loop {
                let mut state = lock.write().unwrap();
                println!("got write lock in receive loop");
                recv(&my_uuid, &mut state);
                sleep(Duration::from_millis(500));
            }
        });
    }
    while let Some(e) = window.next() {
        let mut state = lock.write().unwrap();
        println!("got write lock in render loop");
        match e {
            Input(Press(Keyboard(key))) => match key {
                Key::Left  => state.direction = Direction::Left,
                Key::Right => state.direction = Direction::Right,
                Key::Up    => state.direction = Direction::Up,
                Key::Down  => state.direction = Direction::Down,
                _ => (),
            },
            _ => (),
        };
        if let Some(r) = e.update_args() {
            mov += r.dt;
        }
        if mov >= 1.0/15.0 {
            mov = mov - 1.0/15.0;
            update(&mut state);
        }
        window.draw_2d(&e, |c, g| {
            piston_window::clear([0.0; 4], g);

            let len = (&state.snek_body).len() as f32;
            for (i, segment) in (&state.snek_body).iter().enumerate() {
                piston_window::rectangle(
                    orange(1.0 - (i as f32) / len * 0.9),
                    [segment.0, segment.1, 20.0, 20.0],
                    c.transform,
                    g);
            }
            for wall in &state.walls {
                piston_window::rectangle(
                    blue(1.0),
                    [wall.0, wall.1, 20.0, 20.0],
                    c.transform,
                    g);
            }
            piston_window::ellipse(
                red(1.0),
                [state.apple.0 + 5.0, state.apple.1 + 5.0, 10.0, 10.0],
                c.transform,
                g);

            let len_other = (&state.other_body).len() as f32;
            for (i, segment) in (&state.other_body).iter().enumerate() {
                piston_window::rectangle(
                    green(1.0 - (i as f32) / len * 0.9),
                    [segment.0, segment.1, 20.0, 20.0],
                    c.transform,
                    g);
            }
        });
        println!("render done");
    }
}

fn update(state: &mut State) {
    let mut rng = rand::thread_rng();
    let width_range: Range<f64> = Range::new(5.0, (SCREEN_WIDTH - 10) as f64);
    let height_range: Range<f64> = Range::new(5.0, (SCREEN_HEIGHT - 10) as f64);

    let will_eat_apple;
    let mut eat_tail: Option<usize> = None;
    let new_head = {
        let head = state.snek_body.front().unwrap();
        let delta = shift(&state.direction);
        will_eat_apple = should_eat(&head, &state.apple);
        for (i, segment) in (&state.snek_body).iter().enumerate() {
            if i != 0 && should_eat(&head, &segment) {
                eat_tail = Some(i);
                break;
            }
        }
        (head.0 + delta.0, head.1 + delta.1)
    };

    if let Some(i) = eat_tail {
        let mut eaten_tail = state.snek_body.split_off(i);
        state.walls.append(&mut eaten_tail);
        state.snek_body.pop_back();
    } else if !will_eat_apple {
        state.snek_body.pop_back();
    } else {
        let new_apple_x = (width_range.ind_sample(&mut rng) / CELL_LENGTH).round() * CELL_LENGTH + 10.0;
        let new_apple_y = (height_range.ind_sample(&mut rng) / CELL_LENGTH).round() * CELL_LENGTH + 10.0;
        state.apple = (new_apple_x, new_apple_y);
    }

    state.snek_body.push_front(new_head);
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

fn send(my_uuid: &Uuid, state: &State) -> Result<(), Box<Error>> {
   let socket = {
        if let Ok(sok) = UdpSocket::bind("0.0.0.0:34254") {
            sok
        } else {
            try!(UdpSocket::bind("0.0.0.0:34255"))
        }
    };
    socket.set_broadcast(true);
    // todo: get rid of clone
    let message = Message {id: my_uuid.clone(), body: state.snek_body.clone()};
    let mut serialized = serde_json::to_string(&message).unwrap();
    print!("Serialized: {} | ", serialized);
    serialized.push('\n');
    let bytes = &serialized.into_bytes()[..];
    try!(socket.send_to(bytes, ("255.255.255.255", 34254)));
    try!(socket.send_to(bytes, ("255.255.255.255", 34255)));
    println!("sent");
    return Ok(());
}

fn recv(my_uuid: &Uuid, state: &mut State) -> Result<(), Box<Error>> {
    let socket = {
        if let Ok(sok) = UdpSocket::bind("0.0.0.0:34254") {
            sok
        } else {
            try!(UdpSocket::bind("0.0.0.0:34255"))
        }
    };
    socket.set_read_timeout(Some(Duration::from_millis(1)));
    let data = read_buf(&socket)?;
    let deserialized: Message = serde_json::from_str(&data).unwrap();

    if deserialized.id != *my_uuid {
        println!("Got this point: {:?} from ID: {}",
                 deserialized.body, deserialized.id);
        state.other_body = deserialized.body;
    }
    return Ok(());
}

#[derive(Debug)]
struct SnekError {
}

impl fmt::Display for SnekError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SnekError is here!")
    }
}

impl Error for SnekError {
    fn description(&self) -> &str {
        "Snek error"
    }
}

// TODO:
// * draw the snake after other stuff
// * use cell coordinates
// * make sure that apples don't appear outside of the field
// * get rid of try!
// * use CELL_LENGTH everywhere it should be usedf

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
    body: LinkedList<Point>
}

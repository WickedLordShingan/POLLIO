#![allow(unused)]

use async_stuff::task::keyboard::KeyBoard;
use async_stuff::task::reactor;
use async_stuff::task::{Task, executor::Executor, reactor::Reactor};
use conquer_once::OnceCell;
use futures_util::stream::StreamExt;
use libc::exit;
use termios::*;

async fn gib_number() -> u32 {
    32
}

async fn test() {
    let res = gib_number().await;
    println!("{} is the result", res);
}

fn keycode_to_ascii(code: u16) -> Option<u8> {
    match code {
        2 => Some(b'1'),
        3 => Some(b'2'),
        4 => Some(b'3'),
        5 => Some(b'4'),
        6 => Some(b'5'),
        7 => Some(b'6'),
        8 => Some(b'7'),
        9 => Some(b'8'),
        10 => Some(b'9'),
        11 => Some(b'0'),
        12 => Some(b'-'),
        13 => Some(b'='),
        16 => Some(b'q'),
        17 => Some(b'w'),
        18 => Some(b'e'),
        19 => Some(b'r'),
        20 => Some(b't'),
        21 => Some(b'y'),
        22 => Some(b'u'),
        23 => Some(b'i'),
        24 => Some(b'o'),
        25 => Some(b'p'),
        26 => Some(b'['),
        27 => Some(b']'),
        30 => Some(b'a'),
        31 => Some(b's'),
        32 => Some(b'd'),
        33 => Some(b'f'),
        34 => Some(b'g'),
        35 => Some(b'h'),
        36 => Some(b'j'),
        37 => Some(b'k'),
        38 => Some(b'l'),
        39 => Some(b';'),
        40 => Some(b'\''),
        41 => Some(b'`'),
        43 => Some(b'\\'),
        44 => Some(b'z'),
        45 => Some(b'x'),
        46 => Some(b'c'),
        47 => Some(b'v'),
        48 => Some(b'b'),
        49 => Some(b'n'),
        50 => Some(b'm'),
        51 => Some(b','),
        52 => Some(b'.'),
        53 => Some(b'/'),
        57 => Some(b' '),
        _ => None,
    }
}

async fn keyboard_task() {
    let mut keyboard = KeyBoard::init();
    while let Some(keycode) = keyboard.next().await {
        if let Some(byte) = keycode_to_ascii(keycode) {
            if (byte as char == 'q') {
                unsafe {
                    exit(0);
                }
            }
            println!("here : {}", byte as char);
        } else {
            println!(".");
        }
    }
}

fn main() {
    let mut term = Termios::from_fd(0).expect("couldnt find fd");
    let copy = term;
    term.c_lflag &= !(ICANON | ECHO | ECHOE | ECHOK | ECHONL | ISIG | IEXTEN);
    tcsetattr(0, TCSANOW, &term).unwrap();

    let mut exe = Executor::init();
    reactor::Reactor::init();
    let handle = std::thread::spawn(|| {
        reactor::REACTOR.get().unwrap().react();
    });
    exe.spawn(keyboard_task());
    exe.run();

    tcsetattr(0, TCSANOW, &copy).unwrap();
}

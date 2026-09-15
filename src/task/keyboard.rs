use futures::Stream;
use std::{fs::File, os::fd::AsRawFd};

use libc::{F_GETFL, F_SETFL, O_NONBLOCK, fcntl};
use std::os::unix::io::RawFd;

use libc::{EAGAIN, EWOULDBLOCK, read};
use std::{io, task::Poll};

use crate::task::reactor::REACTOR;
//here is the plan
//the stream would do read() on the keyboard fd
//if the fd returns FD then register waker
//check again if not poll pending
//else ready(byte)
//waker would just insert the task into the taskqueue
//need custom waker now

#[repr(C)]
struct InputEvent {
    tv_sec: i64,
    tv_usec: i64,
    _type: u16,
    code: u16,
    value: i32,
}

const INPUT_EVENT_SIZE: usize = std::mem::size_of::<InputEvent>();
const EVKEY: u16 = 0x01;

pub struct KeyBoard {
    file: File,
    fd: i32,
}

impl KeyBoard {
    pub fn init() -> Self {
        let path = glob::glob("/dev/input/by-path/*-kbd")
            .expect("FAILED TO CONSTURCT GLOB")
            .next()
            .unwrap()
            .expect("FAILED TO FIND KEYBOARD");
        let keyboard = File::open(path).expect("BRUH");
        let keyboard_fd = keyboard.as_raw_fd();

        unsafe {
            let flags = fcntl(keyboard_fd, F_GETFL, 0);
            assert!(flags >= 0, "fcntl F_GETFL failed");
            let res = fcntl(keyboard_fd, F_SETFL, flags | O_NONBLOCK);
            assert!(res >= 0, "fcntl F_SETFL failed");
        }
        KeyBoard {
            file: keyboard,
            fd: keyboard_fd,
        }
    }

    fn try_read_event(&self) -> Option<InputEvent> {
        let mut buf = [0u8; INPUT_EVENT_SIZE];
        let n = unsafe {
            read(
                self.fd,
                buf.as_mut_ptr() as *mut libc::c_void,
                INPUT_EVENT_SIZE,
            )
        };

        if n == INPUT_EVENT_SIZE as isize {
            let event = unsafe { std::ptr::read(buf.as_ptr() as *const InputEvent) };
            Some(event)
        } else if n == 0 {
            // EOF - fd closed on the other end
            panic!("keyboard fd closed");
        } else {
            let err = io::Error::last_os_error();
            match err.raw_os_error() {
                Some(EAGAIN) | Some(EWOULDBLOCK) => None,
                _ => panic!("read failed: {err}"),
            }
        }
    }
}

impl Stream for KeyBoard {
    type Item = u16;
    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        loop {
            match self.try_read_event() {
                Some(event) if event._type == EVKEY && (event.value == 1 || event.value == 2) => {
                    return Poll::Ready(Some(event.code));
                }
                Some(_) => continue,
                None => break,
            }
        }

        REACTOR.get().unwrap().register(self.fd, cx.waker().clone());

        loop {
            match self.try_read_event() {
                Some(event) => {
                    if (event._type == EVKEY && (event.value == 1 || event.value == 2)) {
                        REACTOR.get().unwrap().deregister(self.fd);
                        return Poll::Ready(Some(event.code));
                    } else {
                        continue;
                    }
                }
                None => {
                    return Poll::Pending;
                }
            }
        }
    }
}

use libc::{
    EPOLL_CLOEXEC, EPOLL_CTL_ADD, EPOLL_CTL_DEL, EPOLL_CTL_MOD, EPOLLET, EPOLLIN, epoll_create1,
    epoll_ctl, epoll_event, epoll_wait,
};
use std::collections::HashMap;
use std::os::unix::io::RawFd;
use std::sync::atomic::AtomicU32;
use std::sync::{Mutex, OnceLock};
use std::task::Waker;

pub static REACTOR: OnceLock<Reactor> = OnceLock::new();

pub struct Reactor {
    epoll_fd: RawFd,
    wakers: Mutex<HashMap<RawFd, Waker>>,
}

impl Reactor {
    pub fn init() {
        let epoll_fd = unsafe { epoll_create1(EPOLL_CLOEXEC) };
        assert!(epoll_fd >= 0, "epoll_create1 failed");
        REACTOR
            .set(Reactor {
                epoll_fd,
                wakers: Mutex::new(HashMap::new()),
            })
            .unwrap_or_else(|_| panic!("REACTOR already initialized"));
    }

    pub fn register(&self, fd: RawFd, waker: Waker) {
        static COUNT: AtomicU32 = AtomicU32::new(0);
        COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        // println!("call count : {:?}", COUNT);
        // println!("{:?}", fd);
        let is_new = {
            let mut map = self.wakers.lock().unwrap();
            //for some reason here insertion is not taking place
            map.insert(fd, waker).is_none()
        };
        let mut event = epoll_event {
            events: (EPOLLIN | EPOLLET) as u32,
            u64: fd as u64,
        };
        //we are adding twice
        let op = if is_new { EPOLL_CTL_ADD } else { EPOLL_CTL_MOD };
        let res = unsafe { epoll_ctl(self.epoll_fd, op, fd, &mut event) };
        let error = std::io::Error::last_os_error();
        //get file already exists error
        assert!(res == 0, "epoll_ctl failed");
    }

    pub fn deregister(&self, fd: RawFd) {
        self.wakers.lock().unwrap().remove(&fd);
        let res = unsafe { epoll_ctl(self.epoll_fd, EPOLL_CTL_DEL, fd, std::ptr::null_mut()) };
        assert!(
            res == 0,
            "epoll_ctl DEL failed: {}",
            std::io::Error::last_os_error()
        );
    }

    pub fn react(&self) -> ! {
        let mut events: [epoll_event; 1024] = unsafe { std::mem::zeroed() };
        loop {
            let n =
                unsafe { epoll_wait(self.epoll_fd, events.as_mut_ptr(), events.len() as i32, -1) };
            assert!(n >= 0, "epoll_wait failed");
            for i in 0..n as usize {
                // println!("epoll {n}");
                let fd = events[i].u64 as RawFd;
                if let Some(waker) = self.wakers.lock().unwrap().get(&fd) {
                    waker.wake_by_ref();
                }
            }
        }
    }
}

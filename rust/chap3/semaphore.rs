use std::sync::{Condvar, Mutex};
use std::convert::TryInto;

pub struct Semaphore {
    mutex: Mutex<usize>,
    cond: Condvar,
    max: isize,
}

impl Semaphore {
    pub fn new(max: isize) -> Self {
        Semaphore {
            mutex: Mutex::new(0),
            cond: Condvar::new(),
            max,
        }
    }

    pub fn wait(&self) {
        let mut count = self.mutex.lock().unwrap();
        while *count >= self.max.try_into().unwrap() {
            count = self.cond.wait(count).unwrap();
        }
        *count += 1;
    }

    pub fn post(&self) {
        let mut count = self.mutex.lock().unwrap();
        *count -= 1;
        if *count <= self.max.try_into().unwrap() {
            self.cond.notify_one();
        }
    }
}

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

const NUM_LOOP: usize = 100000;
const NUM_THREADS: usize = 8;
const SEM_NUM: isize = 4;

static mut CNT: AtomicUsize = AtomicUsize::new(0);

fn main() {
    let mut v = Vec::new();
    let sem = Arc::new(Semaphore::new(SEM_NUM));

    for i in 0..NUM_THREADS {
        let s = sem.clone();
        v.push(std::thread::spawn(move || {
            for _ in 0..NUM_LOOP {
                s.wait();

                unsafe { CNT.fetch_add(1, Ordering::SeqCst); }
                let n = unsafe { CNT.load(Ordering::SeqCst) };
                println!("semaphore: i = {}, CNT = {}", i, n);
                assert!((n as isize) <= SEM_NUM);
                unsafe { CNT.fetch_sub(1, Ordering::SeqCst); }

                s.post();
            }
        }));
    }

    for t in v {
        t.join().unwrap();
    }
}
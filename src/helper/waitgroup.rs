use std::{
    fmt,
    sync::{Arc, Condvar, Mutex},
};

#[derive(Clone)]
pub struct WaitGroup(Arc<WaitGroupImpl>);

struct WaitGroupImpl {
    cond: Condvar,
    count: Mutex<usize>,
}

pub fn new() -> WaitGroup {
    WaitGroup(Arc::new(WaitGroupImpl {
        cond: Condvar::new(),
        count: Mutex::new(0),
    }))
}

impl WaitGroup {
    pub fn add(&self, delta: usize) {
        let mut count = self.0.count.lock().unwrap();
        *count += delta;
        self.notify_if_empty(*count);
    }

    pub fn done(&self) {
        let mut count = self.0.count.lock().unwrap();
        if *count == 0 {
            self.notify_if_empty(*count);
            return;
        }

        *count -= 1;
        self.notify_if_empty(*count);
    }

    pub fn wait(&self) {
        let mut count = self.0.count.lock().unwrap();

        while *count > 0 {
            count = self.0.cond.wait(count).unwrap();
        }
    }

    fn notify_if_empty(&self, count: usize) {
        if count == 0 {
            self.0.cond.notify_all();
        }
    }
}

impl fmt::Debug for WaitGroup {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let count = self.0.count.lock().unwrap();
        write!(f, "WaitGroup {{ count {:?} }}", *count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    #[test]
    fn it_work() {
        let wg = new();
        let v = vec![1, 2, 3, 4, 5];
        wg.add(v.len());
        for _ in v {
            let wg = wg.clone();
            thread::spawn(move || wg.done());
        }
        wg.wait();
    }
}

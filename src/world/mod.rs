use std::{thread, time::Duration};

pub mod village;

pub fn idle_for(dur: Duration) {
    thread::sleep(dur)
}

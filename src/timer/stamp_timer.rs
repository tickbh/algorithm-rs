use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::Timer;

pub struct StampTimer<T> {
    duration: Duration,
    is_sec: bool,
    pub val: T,
}

impl<T> StampTimer<T> {
    pub fn new(val: T, duration: Duration) -> Self {
        let is_sec = duration.as_secs() as u128 * 1000 == duration.as_millis();
        Self { val, duration, is_sec }
    }

    pub fn new_second(val: T, duration: Duration) -> Self {
        Self {
            val,
            duration,
            is_sec: true,
        }
    }

    pub fn new_millis(val: T, duration: Duration) -> Self {
        Self {
            val,
            duration,
            is_sec: false,
        }
    }
}

impl<T> Timer for StampTimer<T> {
    fn when(&self) -> u64 {
        let when = SystemTime::now() + self.duration;
        if self.is_sec {
            when.duration_since(UNIX_EPOCH).unwrap().as_secs()
        } else {
            when.duration_since(UNIX_EPOCH).unwrap().as_millis() as u64
        }
    }
}

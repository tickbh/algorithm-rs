
use std::time::SystemTime;

#[inline(always)]
pub fn get_timestamp() -> u64 {
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).expect("ok").as_secs()
}

#[inline(always)]
pub fn get_milltimestamp() -> u128 {
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).expect("ok").as_millis()
}
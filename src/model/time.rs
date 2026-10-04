use std::time::{SystemTime, UNIX_EPOCH};

pub fn seconds_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_clock_is_past_the_epoch() {
        assert!(seconds_now() > 0);
    }
}

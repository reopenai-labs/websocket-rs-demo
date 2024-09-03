pub mod time_util {
    use std::time::{SystemTime, SystemTimeError, UNIX_EPOCH};

    pub fn current_timestamp() -> u128 {
        let now = SystemTime::now();
        now.duration_since(UNIX_EPOCH).unwrap().as_millis()
    }

    pub fn safely_current_timestamp() -> anyhow::Result<u128, SystemTimeError> {
        let now = SystemTime::now();
        now.duration_since(UNIX_EPOCH).map(|time| time.as_millis())
    }
}

pub mod id_util {
    use uuid::Uuid;

    pub fn uuid() -> String {
        Uuid::new_v4().to_string()
    }
}

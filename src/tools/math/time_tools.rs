use std::time::{Duration, Instant};

///converts an duration to an float where one second is one unit, but keeps the nanoseconds as well
pub fn dur_as_f32(duration: Duration) -> f32{
    let secs = duration.as_secs();
    let nanos = duration.subsec_nanos();
    let nanos_frac = nanos as f32 / 1_000_000_000.0;

    secs as f32 + nanos_frac

}

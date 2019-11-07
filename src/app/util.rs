pub struct ScopeTimer<'a> {
    start_time: std::time::Instant,
    name: &'a str,
}

impl<'a> ScopeTimer<'a> {
    #[allow(unused_must_use)]
    pub fn new(name: &'a str) -> Self {
        ScopeTimer {
            start_time: std::time::Instant::now(),
            name,
        }
    }
}

impl<'a> Drop for ScopeTimer<'a> {
    fn drop(&mut self) {
        let end_time = std::time::Instant::now();
        trace!(
            "ScopeTimer {}: {}",
            self.name,
            (end_time - self.start_time).as_micros() as f64 / 1000.0
        )
    }
}

#[derive(Default)]
pub struct PeriodicEvent {
    last_time_triggered: Option<std::time::Instant>,
}

impl PeriodicEvent {
    pub fn try_take_event(
        &mut self,
        current_time: std::time::Instant,
        wait_duration: std::time::Duration,
    ) -> bool {
        match self.last_time_triggered {
            None => {
                self.last_time_triggered = Some(current_time);
                true
            }
            Some(last_time_triggered) => {
                if current_time - last_time_triggered >= wait_duration {
                    self.last_time_triggered = Some(current_time);
                    true
                } else {
                    false
                }
            }
        }
    }
}

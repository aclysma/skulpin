//! Utilities for tracking time in a skulpin App

use std::time;

const NANOS_PER_SEC: u32 = 1_000_000_000;

/// Contains the global time information (such as time when app was started.) There is also a
/// time context that is continuously updated
pub struct TimeState {
    app_start_system_time: time::SystemTime,
    app_start_instant: time::Instant,

    // Save the instant captured during previous update
    previous_update_instant: time::Instant,

    // This contains each context that we support. This will likely be removed in a future version
    // of skulpin
    app_time_context: TimeContext,
}

impl TimeState {
    /// Create a new TimeState. Default is not allowed because the current time affects the object
    #[allow(clippy::new_without_default)]
    pub fn new() -> TimeState {
        let now_instant = time::Instant::now();
        let now_system_time = time::SystemTime::now();

        TimeState {
            app_start_system_time: now_system_time,
            app_start_instant: now_instant,
            previous_update_instant: now_instant,
            app_time_context: TimeContext::new(),
        }
    }

    /// Call every frame to capture time passing and update values
    pub fn update(&mut self) {
        // Determine length of time since last tick
        let now_instant = time::Instant::now();
        let elapsed = now_instant - self.previous_update_instant;
        self.previous_update_instant = now_instant;
        self.app_time_context.update(elapsed);
    }

    /// System time that the application started
    pub fn app_start_system_time(&self) -> &time::SystemTime {
        &self.app_start_system_time
    }

    /// rust Instant object captured when the application started
    pub fn app_start_instant(&self) -> &time::Instant {
        &self.app_start_instant
    }

    /// Get the app time context.
    pub fn app_time_context(&self) -> &TimeContext {
        &self.app_time_context
    }

    /// Duration of time passed
    pub fn total_time(&self) -> time::Duration {
        self.app_time_context.total_time
    }

    /// `std::time::Instant` object captured at the start of the most recent update
    pub fn current_instant(&self) -> time::Instant {
        self.app_time_context.current_instant
    }

    /// duration of time passed during the previous update
    pub fn previous_update_time(&self) -> time::Duration {
        self.app_time_context.previous_update_time
    }

    /// previous update time in f32 seconds
    pub fn previous_update_dt(&self) -> f32 {
        self.app_time_context.previous_update_dt
    }

    /// estimate of updates per second
    pub fn updates_per_second(&self) -> f32 {
        self.app_time_context.updates_per_second
    }

    /// estimate of updates per second smoothed over time
    pub fn updates_per_second_smoothed(&self) -> f32 {
        self.app_time_context.updates_per_second_smoothed
    }

    /// Total number of updates
    pub fn update_count(&self) -> u64 {
        self.app_time_context.update_count
    }
}

/// Tracks time passing, this is separate from the "global" `TimeState` since it would be
/// possible to track a separate "context" of time, for example "unpaused" time in a game
#[derive(Copy, Clone)]
pub struct TimeContext {
    total_time: time::Duration,
    current_instant: time::Instant,
    previous_update_time: time::Duration,
    previous_update_dt: f32,
    updates_per_second: f32,
    updates_per_second_smoothed: f32,
    update_count: u64,
}

impl TimeContext {
    /// Create a new TimeState. Default is not allowed because the current time affects the object
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let now_instant = time::Instant::now();
        let zero_duration = time::Duration::from_secs(0);
        TimeContext {
            total_time: zero_duration,
            current_instant: now_instant,
            previous_update_time: zero_duration,
            previous_update_dt: 0.0,
            updates_per_second: 0.0,
            updates_per_second_smoothed: 0.0,
            update_count: 0,
        }
    }

    /// Call to capture time passing and update values
    pub fn update(
        &mut self,
        elapsed: std::time::Duration,
    ) {
        self.total_time += elapsed;
        self.current_instant += elapsed;
        self.previous_update_time = elapsed;

        // this can eventually be replaced with as_float_secs
        let dt =
            (elapsed.as_secs() as f32) + (elapsed.subsec_nanos() as f32) / (NANOS_PER_SEC as f32);

        self.previous_update_dt = dt;

        let fps = if dt > 0.0 { 1.0 / dt } else { 0.0 };

        //TODO: Replace with a circular buffer
        const SMOOTHING_FACTOR: f32 = 0.95;
        self.updates_per_second = fps;
        self.updates_per_second_smoothed = (self.updates_per_second_smoothed * SMOOTHING_FACTOR)
            + (fps * (1.0 - SMOOTHING_FACTOR));

        self.update_count += 1;
    }

    /// Duration of time passed in this time context
    pub fn total_time(&self) -> time::Duration {
        self.total_time
    }

    /// `std::time::Instant` object captured at the start of the most recent update in this time
    /// context
    pub fn current_instant(&self) -> time::Instant {
        self.current_instant
    }

    /// duration of time passed during the previous update
    pub fn previous_update_time(&self) -> time::Duration {
        self.previous_update_time
    }

    /// previous update time in f32 seconds
    pub fn previous_update_dt(&self) -> f32 {
        self.previous_update_dt
    }

    /// estimate of updates per second
    pub fn updates_per_second(&self) -> f32 {
        self.updates_per_second
    }

    /// estimate of updates per second smoothed over time
    pub fn updates_per_second_smoothed(&self) -> f32 {
        self.updates_per_second_smoothed
    }

    /// Total number of update in this time context
    pub fn update_count(&self) -> u64 {
        self.update_count
    }
}

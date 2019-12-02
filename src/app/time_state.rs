//! Utilities for tracking time in a skuplin App

use std::time;

/// An enum that allows for measuing time in different contexts. This is likely to be removed
/// in a future version of Skulpin
#[derive(Copy, Clone, PartialEq, strum_macros::EnumCount, Debug)]
pub enum TimeContext {
    // Normal system/wallclock time, never stops
    System,
}

// count of TimeContext enum values
const TIME_CONTEXT_COUNT: usize = TIMECONTEXT_COUNT;

const NANOS_PER_SEC: u32 = 1_000_000_000;

//TODO: Avoid using pub for fields
//TODO: Probably doesn't make sense to keep TimeState and ModeTimeState separate

/// Contains the global time information (such as time when app was started)
// This is not intended to be accessed when the system time updates, but we can double buffer it
// if it becomes a problem
pub struct TimeState {
    /// System time that the application started
    pub app_start_system_time: time::SystemTime,

    /// rust Instant object captured when the application started
    pub app_start_instant: time::Instant,

    /// rust Instant object captured at the start of the most recent frame
    pub previous_instant: time::Instant,

    /// The app could have different timers for tracking paused/unpaused time seperately
    /// This is likely to be removed in a future version of Skulpin
    pub previous_time_context: TimeContext,

    // This contains each context that we support. This will likely be removed in a future version
    // of skulpin
    time_context_states: [ModeTimeState; TIME_CONTEXT_COUNT],
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
            previous_instant: now_instant,
            previous_time_context: TimeContext::System,
            time_context_states: [ModeTimeState::new(); TIME_CONTEXT_COUNT],
        }
    }

    /// Call every frame to capture time passing and update values
    pub fn update(
        &mut self,
        time_context: TimeContext,
    ) {
        // Cache the mode we are in this frame
        self.previous_time_context = time_context;

        // Determine length of time since last tick
        let now_instant = time::Instant::now();
        let elapsed = now_instant - self.previous_instant;
        self.previous_instant = now_instant;

        for time_context_index in 0..TIME_CONTEXT_COUNT {
            let mode_elapsed = if time_context_index <= (time_context as usize) {
                elapsed
            } else {
                std::time::Duration::from_secs(0)
            };

            self.time_context_states[time_context_index].update(mode_elapsed);
        }
    }

    /// Get the system time context. This getter will likely be replaced with members of
    /// ModeTimeState
    pub fn system(&self) -> &ModeTimeState {
        &self.time_context_states[TimeContext::System as usize]
    }
}

/// Tracks time passing, this is separate from the "global" `TimeState` since it would be possible
/// to have "system" time, "unpaused" time, etc.
#[derive(Copy, Clone)]
pub struct ModeTimeState {
    /// Duration of time passed in this mode
    pub total_time: time::Duration,

    /// rust Instant object captured at the start of the most recent frame in this mode
    pub frame_start_instant: time::Instant,

    /// duration of time passed during the previous frame
    pub previous_frame_time: time::Duration,

    /// previous frame time in f32
    pub previous_frame_dt: f32,

    /// estimate of frame rate
    pub fps: f32,

    /// estimate of frame rate smoothed over time
    pub fps_smoothed: f32,

    /// Total number of frames in this mode
    pub frame_count: u64,
}

impl ModeTimeState {
    /// Create a new TimeState. Default is not allowed because the current time affects the object
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let now_instant = time::Instant::now();
        let zero_duration = time::Duration::from_secs(0);
        ModeTimeState {
            total_time: zero_duration,
            frame_start_instant: now_instant,
            previous_frame_time: zero_duration,
            previous_frame_dt: 0.0,
            fps: 0.0,
            fps_smoothed: 0.0,
            frame_count: 0,
        }
    }

    /// Call every frame to capture time passing and update values
    pub fn update(
        &mut self,
        elapsed: std::time::Duration,
    ) {
        self.total_time += elapsed;
        self.frame_start_instant += elapsed;
        self.previous_frame_time = elapsed;

        // this can eventually be replaced with as_float_secs
        let dt =
            (elapsed.as_secs() as f32) + (elapsed.subsec_nanos() as f32) / (NANOS_PER_SEC as f32);

        self.previous_frame_dt = dt;

        let fps = if dt > 0.0 { 1.0 / dt } else { 0.0 };

        //TODO: Replace with a circular buffer
        const SMOOTHING_FACTOR: f32 = 0.95;
        self.fps = fps;
        self.fps_smoothed =
            (self.fps_smoothed * SMOOTHING_FACTOR) + (fps * (1.0 - SMOOTHING_FACTOR));

        self.frame_count += 1;
    }
}

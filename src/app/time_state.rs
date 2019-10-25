use std::time;

#[derive(Copy, Clone, PartialEq, strum_macros::EnumCount, Debug)]
pub enum TimeContext {
    // Normal system/wallclock time, never stops
    System,
}

const TIME_CONTEXT_COUNT : usize = TIMECONTEXT_COUNT;

const NANOS_PER_SEC: u32 = 1_000_000_000;

//TODO: Exposing duration/instant is a little dangerous, it would be better if durations/instants
// from each mode were different types, and couldn't be used directly with stdlib duration/instants

//TODO: Avoid using pub for fields

// This is not intended to be accessed when the system time updates, but we can double buffer it
// if it becomes a problem
pub struct TimeState {
    // System time that the application started
    pub app_start_system_time: time::SystemTime,

    // rust Instant object captured when the application started
    pub app_start_instant: time::Instant,

    // rust Instant object captured at the start of the frame
    pub previous_instant: time::Instant,

    // The app could have different timers for tracking paused/unpaused time seperately
    pub previous_time_context: TimeContext,

    time_context_states: [ModeTimeState; TIME_CONTEXT_COUNT],
}

impl Default for TimeState {
    fn default() -> TimeState {
        let now_instant = time::Instant::now();
        let now_system_time = time::SystemTime::now();

        return TimeState {
            app_start_system_time: now_system_time,
            app_start_instant: now_instant,
            previous_instant: now_instant,
            previous_time_context: TimeContext::System,
            time_context_states: [ModeTimeState::new(); TIME_CONTEXT_COUNT],
        };
    }
}

impl TimeState {
    pub fn update(&mut self, time_context: TimeContext) {
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

//        trace!(
//            "fps: {:.1}  dt: {:.2}ms",
//            self.time_context_states[0].fps,
//            self.time_context_states[0].previous_frame_dt * 1000.0
//        );

        if self.time_context_states[0].previous_frame_dt > 1.0 / 30.0 {
            //warn!("slow frame (dt: {:.2}ms)", dt);
        }
    }

    pub fn system(&self) -> &ModeTimeState {
        &self.time_context_states[TimeContext::System as usize]
    }
}

#[derive(Copy, Clone)]
pub struct ModeTimeState {
    // Duration of time passed since app_start_system_time
    pub total_time: time::Duration,

    // rust Instant object captured at the start of the frame
    pub frame_start_instant: time::Instant,

    // duration of time passed during the previous frame
    pub previous_frame_time: time::Duration,

    pub previous_frame_dt: f32,

    pub fps: f32,

    pub fps_smoothed: f32,

    pub frame_count: u64,
}

impl ModeTimeState {
    pub fn new() -> Self {
        let now_instant = time::Instant::now();
        let zero_duration = time::Duration::from_secs(0);
        return ModeTimeState {
            total_time: zero_duration,
            frame_start_instant: now_instant,
            previous_frame_time: zero_duration,
            previous_frame_dt: 0.0,
            fps: 0.0,
            fps_smoothed: 0.0,
            frame_count: 0,
        };
    }

    pub fn update(&mut self, elapsed: std::time::Duration) {
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

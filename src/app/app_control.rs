/// State that drives high-level decision making for the app
#[derive(Default)]
pub struct AppControl {
    /// If true, the application will quit when the next frame ends
    should_terminate_process: bool,
}

impl AppControl {
    /// Direct the application to terminate at the end of the next frame
    pub fn enqueue_terminate_process(&mut self) {
        self.should_terminate_process = true;
    }

    /// Returns true iff `enqueue_terminate_process` is called, indicating that the app should terminate
    pub fn should_terminate_process(&self) -> bool {
        self.should_terminate_process
    }
}

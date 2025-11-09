use crate::Model;

pub mod breakpoint;
pub mod buf_logger;
pub mod vcd_logger;

pub use breakpoint::BreakPoint;
pub use buf_logger::BufLogger;
pub use vcd_logger::VCDLoggerHook;

// Hook trait for extending simulator behavior
pub trait Hook: Send {
    /// Called at each simulation step
    fn on_step(&mut self, _time: u64, _model: &Model) {}

    /// Called before clock edge
    fn pre_clock(&mut self, _time: u64, _clock_name: &str, _model: &Model) {}

    /// Called after clock edge
    fn post_clock(&mut self, _time: u64, _clock_name: &str, _model: &Model) {}

    /// Called at reset
    fn on_reset(&mut self, _time: u64, _model: &Model) {}

    /// Called at simulation end
    fn on_finish(&mut self, _time: u64, _model: &Model) {}
}

use super::Hook;

// This hook traps the simulation when a specific condition is met
// useful for debugging
pub struct BreakPoint {
    // TODO: Implement conditional breakpoints
}

impl BreakPoint {
    #[allow(dead_code)]
    pub fn new() -> Self {
        BreakPoint {}
    }
}

impl Hook for BreakPoint {
    // TODO: Implement hook methods for conditional breakpoints
}

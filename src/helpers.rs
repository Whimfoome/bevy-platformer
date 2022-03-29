use bevy::core::Timer;

pub trait TimerHelper {
    fn start(&mut self);
    fn is_stopped(&mut self) -> bool;
}

impl TimerHelper for Timer {
    fn start(&mut self) {
        self.reset();
        self.unpause();
    }

    fn is_stopped(&mut self) -> bool {
        return self.paused() || self.finished();
    }
}
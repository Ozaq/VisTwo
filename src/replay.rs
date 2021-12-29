use crate::legacy_parsers::Frame;
use crate::legacy_parsers::Trajectory;
use std::cmp;
use std::time::Duration;

#[derive(Debug)]
pub struct Replay {
    trajectory: Trajectory,
    pub current_frame_index: usize,
    frame_duration: Duration,
    elapsed: Duration,
    total_duration: Duration,
}

impl Replay {
    pub fn new(trajectory: Trajectory, frame_duration: Duration) -> Self {
        let frame_count = trajectory.frames.len();
        let total_duration = if frame_count == 0 {
            Duration::from_secs(0)
        } else {
            frame_duration * (frame_count - 1) as u32
        };
        Self {
            trajectory,
            current_frame_index: 0,
            frame_duration,
            elapsed: Duration::from_secs(0),
            total_duration,
        }
    }

    pub fn advance_by(&mut self, duration: Duration) {
        self.elapsed = cmp::min(self.total_duration, self.elapsed + duration);
        self.current_frame_index =
            (self.elapsed.as_secs_f64() / self.frame_duration.as_secs_f64()) as usize;
    }

    pub fn current_frame(&self) -> &Frame {
        &self.trajectory.frames[self.current_frame_index]
    }

    pub fn area(&self) -> (f32, f32, f32, f32) {
        self.trajectory.area()
    }

    pub fn frames(&self) -> usize {
        self.trajectory.frames.len()
    }
}

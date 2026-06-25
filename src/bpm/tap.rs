use std::collections::VecDeque;
use std::time::Instant;

pub struct TapDetector {
    timestamps: VecDeque<Instant>,
    max_taps: usize,
    timeout_secs: f64,
}

impl TapDetector {
    pub fn new(max_taps: usize, timeout_secs: f64) -> Self {
        Self {
            timestamps: VecDeque::with_capacity(max_taps + 1),
            max_taps,
            timeout_secs,
        }
    }

    pub fn tap(&mut self) -> Option<f64> {
        let now = Instant::now();

        if let Some(&last) = self.timestamps.back() {
            if now.duration_since(last).as_secs_f64() > self.timeout_secs {
                self.timestamps.clear();
            }
        }

        self.timestamps.push_back(now);

        if self.timestamps.len() > self.max_taps {
            self.timestamps.pop_front();
        }

        self.current_bpm()
    }

    pub fn reset(&mut self) {
        self.timestamps.clear();
    }

    pub fn current_bpm(&self) -> Option<f64> {
        if self.timestamps.len() < 2 {
            return None;
        }

        let intervals: Vec<f64> = self
            .timestamps
            .iter()
            .zip(self.timestamps.iter().skip(1))
            .map(|(a, b)| b.duration_since(*a).as_secs_f64())
            .collect();

        let avg_interval = intervals.iter().sum::<f64>() / intervals.len() as f64;

        if avg_interval > 0.0 {
            Some(60.0 / avg_interval)
        } else {
            None
        }
    }
}

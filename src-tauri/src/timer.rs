use std::time::{Duration, Instant};

/// 倒计时所处阶段
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Idle,
    Running,
    Paused,
    Finished,
}

impl Phase {
    pub fn as_str(self) -> &'static str {
        match self {
            Phase::Idle => "idle",
            Phase::Running => "running",
            Phase::Paused => "paused",
            Phase::Finished => "finished",
        }
    }
}

/// 倒计时状态机。所有依赖时间的计算都接受注入的 `now`，便于单元测试。
pub struct Timer {
    phase: Phase,
    /// Running 状态下的到期时刻
    deadline: Option<Instant>,
    /// Paused 状态下冻结的剩余时长
    remaining: Duration,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            phase: Phase::Idle,
            deadline: None,
            remaining: Duration::ZERO,
        }
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn start(&mut self, seconds: u64) {
        self.start_at(Instant::now(), seconds);
    }

    fn start_at(&mut self, now: Instant, seconds: u64) {
        let total = Duration::from_secs(seconds);
        self.deadline = Some(now + total);
        self.remaining = total;
        self.phase = Phase::Running;
    }

    pub fn pause(&mut self) -> bool {
        self.pause_at(Instant::now())
    }

    fn pause_at(&mut self, now: Instant) -> bool {
        if self.phase != Phase::Running {
            return false;
        }
        self.remaining = self.remaining_at(now);
        self.deadline = None;
        self.phase = Phase::Paused;
        true
    }

    pub fn resume(&mut self) -> bool {
        self.resume_at(Instant::now())
    }

    fn resume_at(&mut self, now: Instant) -> bool {
        if self.phase != Phase::Paused {
            return false;
        }
        self.deadline = Some(now + self.remaining);
        self.phase = Phase::Running;
        true
    }

    pub fn cancel(&mut self) {
        self.phase = Phase::Idle;
        self.deadline = None;
        self.remaining = Duration::ZERO;
    }

    pub fn finish(&mut self) {
        self.phase = Phase::Finished;
        self.deadline = None;
        self.remaining = Duration::ZERO;
    }

    pub fn remaining(&self) -> Duration {
        self.remaining_at(Instant::now())
    }

    pub fn remaining_at(&self, now: Instant) -> Duration {
        match self.phase {
            Phase::Running => self
                .deadline
                .map(|deadline| deadline.saturating_duration_since(now))
                .unwrap_or(Duration::ZERO),
            Phase::Paused => self.remaining,
            Phase::Idle | Phase::Finished => Duration::ZERO,
        }
    }

    pub fn remaining_ms(&self) -> u64 {
        self.remaining().as_millis() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn running_counts_down() {
        let base = Instant::now();
        let mut timer = Timer::new();
        timer.start_at(base, 10);

        assert_eq!(timer.phase(), Phase::Running);
        assert_eq!(timer.remaining_at(base + Duration::from_millis(3000)), Duration::from_millis(7000));
    }

    #[test]
    fn expired_clamps_to_zero() {
        let base = Instant::now();
        let mut timer = Timer::new();
        timer.start_at(base, 2);

        assert_eq!(timer.remaining_at(base + Duration::from_secs(11)), Duration::ZERO);
        assert_eq!(timer.remaining_at(base + Duration::from_secs(2)), Duration::ZERO);
    }

    #[test]
    fn pause_freezes_and_resume_shifts_deadline() {
        let base = Instant::now();
        let mut timer = Timer::new();
        timer.start_at(base, 10);

        assert!(timer.pause_at(base + Duration::from_millis(3000)));
        assert_eq!(timer.phase(), Phase::Paused);
        // 暂停 27 秒后剩余时长不变
        assert_eq!(timer.remaining_at(base + Duration::from_secs(30)), Duration::from_millis(7000));

        assert!(timer.resume_at(base + Duration::from_secs(30)));
        assert_eq!(timer.phase(), Phase::Running);
        // 新的到期时刻 = 继续时刻 + 7 秒
        assert_eq!(timer.remaining_at(base + Duration::from_secs(34)), Duration::from_secs(3));
    }

    #[test]
    fn cancel_returns_to_idle() {
        let base = Instant::now();
        let mut timer = Timer::new();
        timer.start_at(base, 10);
        timer.pause_at(base + Duration::from_secs(1));
        timer.cancel();

        assert_eq!(timer.phase(), Phase::Idle);
        assert_eq!(timer.remaining_at(base + Duration::from_secs(1)), Duration::ZERO);
        assert!(!timer.resume());
    }
}

use super::*;

pub type Time = std::time::Duration;

pub struct Timer {
    pub time: Time,
    pub paused: bool,
    pub mode: TimerMode,
}

#[derive(Clone, Copy)]
pub enum TimerMode {
    Idle,
    Countdown,
    Countup,
}

impl Timer {
    pub fn from_status() -> Result<Self, Box<dyn std::error::Error>> {
        let time = Timer::parse_duration(&std::fs::read_to_string(format!(
            "status/{}.txt",
            TimerBot::NAME
        ))?)?;
        Ok(Self {
            time,
            paused: true,
            mode: TimerMode::Idle,
        })
    }

    pub fn update(&mut self, delta_time: f32) {
        if !self.paused {
            let delta = std::time::Duration::from_secs_f32(delta_time);
            match self.mode {
                TimerMode::Idle => (),
                TimerMode::Countdown => {
                    self.time = self.time.checked_sub(delta).unwrap_or_default();
                }
                TimerMode::Countup => {
                    self.time = self.time.checked_add(delta).unwrap_or_default();
                }
            }
        }
    }

    pub fn time_status(&self) -> String {
        Timer::format_duration(self.time).to_string()
    }

    pub fn parse_duration(s: &str) -> Result<std::time::Duration, Box<dyn std::error::Error>> {
        let mut secs = 0;
        let mut multiplier = 1;
        for t in s.split(':').rev() {
            secs += t.parse::<u64>()? * multiplier;
            multiplier *= 60;
        }
        Ok(std::time::Duration::from_secs(secs))
    }

    pub fn format_duration(duration: std::time::Duration) -> String {
        let secs = duration.as_secs();
        let seconds = secs % 60;
        let minutes = (secs / 60) % 60;
        let hours = (secs / 60 / 60) % 60;
        let mut result = String::new();
        if hours > 0 {
            result.push_str(&format!("{:02}:", hours));
        }
        if hours > 0 || minutes > 0 {
            result.push_str(&format!("{:02}:", minutes));
        }
        result.push_str(&format!("{:02}", seconds));
        result
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            paused: true,
            time: std::time::Duration::from_secs(0),
            mode: TimerMode::Idle,
        }
    }
}

impl Display for TimerMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimerMode::Idle => write!(f, "Paused"),
            TimerMode::Countdown => write!(f, "Count down"),
            TimerMode::Countup => write!(f, "Count up"),
        }
    }
}

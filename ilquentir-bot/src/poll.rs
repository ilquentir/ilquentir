use time::{Duration, OffsetDateTime, Time};
use tracing::error;

/// Describes possible kind of polls
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, sqlx::Type)]
#[sqlx(rename_all = "snake_case")]
pub enum PollKind {
    /// Ask user, how was his day
    HowWasYourDay,
    /// Was there any
    FoodAllergy,
}

impl ToString for PollKind {
    fn to_string(&self) -> String {
        match self {
            Self::HowWasYourDay => "how_was_your_day".to_owned(),
            Self::FoodAllergy => "food_allergy".to_owned(),
        }
    }
}

impl PollKind {
    fn send_at(self) -> Option<Time> {
        match self {
            // 22:00 MSK
            Self::HowWasYourDay => Some(Time::from_hms(19, 0, 0).unwrap()),
            // 21:00 MSK
            Self::FoodAllergy => Some(Time::from_hms(18, 0, 0).unwrap()),
        }
    }

    fn time_between(self) -> Option<Duration> {
        None
    }

    pub fn schedule_next(self, current: OffsetDateTime) -> OffsetDateTime {
        if let Some(send_at) = self.send_at() {
            let next = current.replace_time(send_at);
            if next > current {
                next
            } else {
                next + Duration::DAY
            }
        } else if let Some(time_between) = self.time_between() {
            current + time_between
        } else {
            error!(poll_kind = ?self, %current, "poll kind with unimplemented either sending time or time between");
            panic!("invalid PollKind implementation");
        }
    }

    pub fn question(self) -> String {
        match self {
            Self::HowWasYourDay => "How was your day?",
            Self::FoodAllergy => {
                "Had you encountered any of described feelings after the meal today?"
            }
        }
        .to_owned()
    }

    pub fn allows_multiple_answers(self) -> bool {
        match self {
            Self::HowWasYourDay => false,
            Self::FoodAllergy => true,
        }
    }

    pub fn options(self) -> Vec<String> {
        match self {
            Self::HowWasYourDay => &["+2 (perfect)", "+1", "0", "-1", "-2 (terrible)"][..],
            Self::FoodAllergy => &[
                "Shortness of breath",
                "Itching",
                "Bloating",
                "Nope, nothing :)",
            ][..],
        }
        .iter()
        .map(|&option| option.to_string())
        .collect()
    }
}

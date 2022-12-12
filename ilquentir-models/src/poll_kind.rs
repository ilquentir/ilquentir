use time::{macros::time, Duration, OffsetDateTime, Time};
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
            // 22:00 MSK = 19:00 UTC
            Self::HowWasYourDay => Some(time!(19:00)),
            // 21:00 MSK = 18:00 UTC
            Self::FoodAllergy => Some(time!(18:00)),
        }
    }

    fn time_between(self) -> Option<Duration> {
        None
    }

    /// Get next time to send the poll.
    ///
    /// ## Human-friendliness
    ///
    /// If the next poll is nearer that 12 hours – one day added to the resulting time
    ///
    /// ```rust
    /// # use time::macros::datetime;
    /// assert_eq!(
    ///     PollKind::HowWasYourDay.schedule_next(datetime!(2022-12-05 00:00 UTC)),
    ///     datetime!(2022-12-05 19:00 UTC),
    /// );
    pub fn schedule_next(self, current: OffsetDateTime) -> OffsetDateTime {
        if let Some(send_at) = self.send_at() {
            let next = current.replace_time(send_at);
            if next >= current + Duration::hours(12) {
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

#[cfg(test)]
mod tests {
    use time::macros::datetime;

    use super::*;

    #[test]
    fn test_next_time() {
        for kind in [PollKind::HowWasYourDay, PollKind::FoodAllergy] {
            // match is here to be sure that it's impossible
            // to add new enum kind without testing it :)
            match kind {
                PollKind::HowWasYourDay => {
                    // every day at 19:00 UTC

                    assert_eq!(
                        kind.schedule_next(datetime!(2020-01-01 00:00 UTC)),
                        datetime!(2020-01-01 19:00 UTC)
                    );
                    assert_eq!(
                        kind.schedule_next(datetime!(2020-01-01 07:00 UTC)),
                        datetime!(2020-01-01 19:00 UTC)
                    );
                    assert_eq!(
                        kind.schedule_next(datetime!(2020-01-01 18:00 UTC)),
                        datetime!(2020-01-02 19:00 UTC)
                    );
                    assert_eq!(
                        kind.schedule_next(datetime!(2020-01-01 19:00 UTC)),
                        datetime!(2020-01-02 19:00 UTC)
                    );
                    assert_eq!(
                        kind.schedule_next(datetime!(2020-01-01 19:01 UTC)),
                        datetime!(2020-01-02 19:00 UTC)
                    );
                }
                PollKind::FoodAllergy => {
                    // every day at 18:00 UTC

                    assert_eq!(
                        kind.schedule_next(datetime!(2020-01-01 00:00 UTC)),
                        datetime!(2020-01-01 18:00 UTC)
                    );
                    assert_eq!(
                        kind.schedule_next(datetime!(2020-01-01 06:00 UTC)),
                        datetime!(2020-01-01 18:00 UTC)
                    );
                    assert_eq!(
                        kind.schedule_next(datetime!(2020-01-01 07:00 UTC)),
                        datetime!(2020-01-02 18:00 UTC)
                    );
                    assert_eq!(
                        kind.schedule_next(datetime!(2020-01-01 18:00 UTC)),
                        datetime!(2020-01-02 18:00 UTC)
                    );
                    assert_eq!(
                        kind.schedule_next(datetime!(2020-01-01 19:00 UTC)),
                        datetime!(2020-01-02 18:00 UTC)
                    );
                    assert_eq!(
                        kind.schedule_next(datetime!(2020-01-01 19:01 UTC)),
                        datetime!(2020-01-02 18:00 UTC)
                    );
                }
            }
        }
    }
}
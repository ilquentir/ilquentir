use indexmap::IndexMap;
use teloxide::types::InlineKeyboardButton;

#[derive(Debug, Clone, Copy, strum::EnumString, strum::IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum Scope {
    DailyEvents,
    PromoDailyEvents,
}

impl Scope {
    pub fn to_str(self) -> &'static str {
        match self {
            Self::PromoDailyEvents => "promo_daily",
            _ => self.into(),
        }
    }

    pub fn from_payload(data: &str) -> Option<Self> {
        // TODO: use strum parsing
        match data.split_once(':')?.0 {
            "daily_events" => Some(Self::DailyEvents),
            "promo_daily" => Some(Self::PromoDailyEvents),
            _ => {
                warn!(data, "payload with unknown scope");

                None
            }
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct CallbackButtonData {
    pub value: String,
    payload: String,
    #[allow(dead_code)]
    scope: Scope,
}

impl CallbackButtonData {
    pub fn new(data: impl AsRef<str>, scope: Scope) -> Self {
        Self {
            value: data.as_ref().to_string(),
            payload: format_payload(stable_hash::fast_stable_hash(&data.as_ref()), scope),
            scope,
        }
    }

    pub fn create_button(&self, button_text: impl AsRef<str>) -> InlineKeyboardButton {
        InlineKeyboardButton::callback(button_text.as_ref(), &self.payload)
    }

    pub fn value(&self) -> &String {
        &self.value
    }

    pub fn matches(&self, payload: impl AsRef<str>) -> bool {
        self.payload == payload.as_ref()
    }
}

pub(super) fn make_callback_data(
    options: impl Iterator<Item = impl AsRef<str>>,
    scope: Scope,
) -> IndexMap<String, CallbackButtonData> {
    options
        .map(|option| {
            let data = CallbackButtonData::new(option, scope);

            (data.value.clone(), data)
        })
        .collect()
}

pub(super) fn format_payload(hash: u128, scope: Scope) -> String {
    let res = format!("{scope}:{hash}", scope = scope.to_str(), hash = hash);
    debug_assert!(res.len() <= 64);

    res
}

macro_rules! buttons_row {
    [$([$text:expr, $data:expr]),*] => {
        vec![$($data.create_button($text), )*]
    };
}

pub(super) use buttons_row;
use tracing::warn;

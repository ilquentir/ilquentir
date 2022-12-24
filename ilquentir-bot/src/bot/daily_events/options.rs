use indexmap::IndexMap;
use once_cell::sync::Lazy;

use crate::bot::callbacks::{make_callback_data, CallbackButtonData, Scope};

pub(super) static ALL_OPTIONS: Lazy<IndexMap<String, CallbackButtonData>> = Lazy::new(|| {
    make_callback_data(
        [
            "Выход на улицу (>10 минут)",
            "Сон >6 часов",
            "Общение вживую",
            "Спорт/активность",
            "Хобби/обучение/свой проект",
            "Позитивные личные события",
            "Конфликты с людьми (в том числе онлайн)",
            "Важные новости о внешнем мире",
            "Контакты с семьёй",
            "Солнечно на улице",
            "Стресс на работе",
            "Секс",
            "Алкоголь",
            "Вещества",
            "Поездка за город/путешествие",
            "Проблемы со здоровьем",
        ]
        .into_iter(),
        Scope::DailyEvents,
    )
});

pub(super) static NONE_BUTTON: Lazy<CallbackButtonData> =
    Lazy::new(|| CallbackButtonData::new("none", Scope::DailyEvents));

pub(super) static ALL_BUTTON: Lazy<CallbackButtonData> =
    Lazy::new(|| CallbackButtonData::new("all", Scope::DailyEvents));

pub(super) static DONE_BUTTON: Lazy<CallbackButtonData> =
    Lazy::new(|| CallbackButtonData::new("done", Scope::DailyEvents));

pub(super) static PROMO_YES_BUTTON: Lazy<CallbackButtonData> =
    Lazy::new(|| CallbackButtonData::new("promo_yes", Scope::PromoDailyEvents));

pub(super) static PROMO_NO_BUTTON: Lazy<CallbackButtonData> =
    Lazy::new(|| CallbackButtonData::new("promo_no", Scope::PromoDailyEvents));

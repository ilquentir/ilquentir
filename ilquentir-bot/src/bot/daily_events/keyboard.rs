use color_eyre::Result;
use teloxide::types::InlineKeyboardMarkup;

use ilquentir_models::{PgTransaction, PollCustomOptions, PollKind};

use crate::bot::callbacks::buttons_row;

use super::options;

fn format_option(option: &str, enabled: bool) -> String {
    format!("{} {option}", if enabled { ENABLED } else { DISABLED })
}

const ENABLED: char = '✅';
const DISABLED: char = '⬜';

#[tracing::instrument(skip(txn), err)]
pub async fn user_daily_options(
    txn: &mut PgTransaction<'_>,
    chat_id: i64,
) -> Result<InlineKeyboardMarkup> {
    let current = PollCustomOptions::get_for_user(txn, chat_id, PollKind::DailyEvents).await?;
    let rendered_options = options::ALL_OPTIONS
        .values()
        .map(|data| (data, current.options.contains(&data.value)))
        .map(|(data, enabled)| buttons_row![[format_option(data.value(), enabled), data]])
        .chain([
            buttons_row![
                ["🚫 Ничего из этого", options::NONE_BUTTON],
                ["✅✅✅ Всё", options::ALL_BUTTON]
            ],
            buttons_row![["Сохранить выбор", options::DONE_BUTTON]],
        ]);

    Ok(InlineKeyboardMarkup::new(rendered_options))
}

pub fn promo() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new([buttons_row![
        ["Да", options::PROMO_YES_BUTTON],
        ["Нет", options::PROMO_NO_BUTTON]
    ]])
}

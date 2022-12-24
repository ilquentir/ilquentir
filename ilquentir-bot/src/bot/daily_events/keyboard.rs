use color_eyre::Result;
use teloxide::types::InlineKeyboardMarkup;

use ilquentir_models::{PgTransaction, PollCustomOptions, PollKind};

use crate::bot::callbacks::buttons;

use super::options;

fn format_option(option: &str, enabled: bool) -> String {
    format!("{} {option}", if enabled { ENABLED } else { DISABLED })
}

const ENABLED: char = '✅';
const DISABLED: char = '⬜';

#[tracing::instrument(skip(txn), err)]
pub async fn make_keyboard(
    txn: &mut PgTransaction<'_>,
    chat_id: i64,
) -> Result<InlineKeyboardMarkup> {
    let current = PollCustomOptions::get_for_user(txn, chat_id, PollKind::DailyEvents).await?;
    let rendered_options = options::ALL_OPTIONS
        .values()
        .map(|data| (data, current.options.contains(&data.value)))
        .map(|(data, enabled)| buttons![[format_option(data.value(), enabled), data]])
        .chain(vec![
            buttons![
                ["🚫 Ничего из этого", options::NONE_BUTTON],
                ["✅✅✅ Спрашивать всё", options::ALL_BUTTON]
            ],
            buttons![["Сохранить выбор", options::DONE_BUTTON]],
        ]);

    Ok(InlineKeyboardMarkup::new(rendered_options))
}

pub fn make_promo_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new([
        buttons![
            ["Да", options::PROMO_YES_BUTTON],
            ["Нет", options::PROMO_NO_BUTTON]
        ]
    ])
}

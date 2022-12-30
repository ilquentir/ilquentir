use teloxide::types::{ButtonRequest, KeyboardButton, KeyboardMarkup, WebAppInfo};

pub fn create_timepicker_keyboard() -> KeyboardMarkup {
    let button = KeyboardButton::new("Выбрать время ежедневного опроса").request(
        ButtonRequest::WebApp(WebAppInfo {
            url: "https://expented.github.io/tgdtp/?hide=date&text=SELECT%20TIME"
                .parse()
                .unwrap(),
        }),
    );

    KeyboardMarkup::new([[button]])
        .one_time_keyboard(true)
        .resize_keyboard(true)
}

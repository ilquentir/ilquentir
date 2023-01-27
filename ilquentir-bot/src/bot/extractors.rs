use teloxide::types::{
    MediaKind, MediaText, Message, MessageKind, MessageWebAppData, Poll, Update, UpdateKind,
};

pub(super) fn get_message_text(msg: Message) -> Option<MediaText> {
    if let MessageKind::Common(msg) = msg.kind {
        if let MediaKind::Text(text) = msg.media_kind {
            return Some(text);
        }
    }

    None
}

pub(super) fn get_web_app_data(msg: Message) -> Option<MessageWebAppData> {
    match msg.kind {
        MessageKind::WebAppData(data) => Some(data),
        _ => None,
    }
}

pub(super) fn get_poll(update: Update) -> Option<Poll> {
    match update.kind {
        UpdateKind::Poll(poll) => Some(poll),
        _ => None,
    }
}

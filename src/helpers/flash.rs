use serde::{Deserialize, Serialize};
use tower_sessions::Session;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlashMessage {
    pub kind: String,
    pub message: String,
}

pub async fn set_flash(
    session: &Session,
    kind: &str,
    message: &str,
) -> Result<(), tower_sessions::session::Error> {
    let flash = FlashMessage {
        kind: kind.to_string(),
        message: message.to_string(),
    };
    session.insert("flash_message", flash).await
}

pub async fn get_flash(session: &Session) -> Option<FlashMessage> {
    let flash: Option<FlashMessage> = session.get("flash_message").await.ok().flatten();
    if flash.is_some() {
        session.remove::<FlashMessage>("flash_message").await.ok();
    }
    flash
}

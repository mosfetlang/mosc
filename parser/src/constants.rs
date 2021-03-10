use std::sync::Arc;

use lazy_static::lazy_static;

lazy_static! {
    pub static ref LOG_CODE_TITLE: Arc<String> = Arc::new("Code".to_string());
    pub static ref LOG_HINT_TITLE: Arc<String> = Arc::new("Hint".to_string());
    pub static ref LOG_WARNING_ID_TITLE: Arc<String> = Arc::new("WID".to_string());
    pub static ref LOG_ERROR_ID_TITLE: Arc<String> = Arc::new("EID".to_string());
}

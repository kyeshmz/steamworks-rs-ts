use napi::{Error, Result};
use std::sync::Mutex;

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Mode {
    Client,
    Server,
}

pub(crate) fn initialize<T>(mode: Mode, init: impl FnOnce() -> Result<T>) -> Result<T> {
    static MODE: Mutex<Option<Mode>> = Mutex::new(None);
    let mut active = MODE
        .lock()
        .map_err(|_| Error::from_reason("Steam process initialization interrupted"))?;
    if active.is_some_and(|current| current != mode) {
        return Err(Error::from_reason(
            "Initialize the Steam client and dedicated server in separate processes",
        ));
    }
    let initialized = init()?;
    *active = Some(mode);
    Ok(initialized)
}

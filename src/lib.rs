use napi::bindgen_prelude::Error;
use napi_derive::napi;
use steamworks::AppId;

pub mod client;

#[macro_use]
extern crate lazy_static;

#[napi]
pub fn init(app_id: Option<u32>) -> Result<(), Error> {
    client::init(app_id)
}

#[napi]
pub fn restart_app_if_necessary(app_id: u32) -> bool {
    steamworks::restart_app_if_necessary(AppId(app_id))
}

#[napi]
pub fn run_callbacks() {
    // All Node workers share Steam's manual-dispatch pipe and its last callback buffer.
    static PUMP: std::sync::Mutex<()> = std::sync::Mutex::new(());
    if let Ok(_pump) = PUMP.try_lock() {
        client::get_client().run_callbacks();
    }
}

pub mod api;

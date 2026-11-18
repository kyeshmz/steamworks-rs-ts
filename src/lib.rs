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
    api::runtime::dispatch();
}

pub mod api;

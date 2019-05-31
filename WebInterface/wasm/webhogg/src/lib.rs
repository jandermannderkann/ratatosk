mod client_logger;
mod webhogg_exception;
mod page;
mod canvas;
mod app;

use wasm_bindgen::prelude::*;
use app::WebhoggApplication as App;
use web_sys::Worker;

#[macro_use]
extern crate log;

fn run_application() {
    match App::new().and_then(|app| app.run()) {
        Ok(_) => info!("program terminated successfully"),
        Err(e) => error!("program terminated with failure > {}", e)
    }
}

#[wasm_bindgen]
pub fn game_logic_entry(worker: Worker) {
    client_logger::init_logger();

    info!("game logic initialisation");
}

#[wasm_bindgen]
pub fn graphics_entry(worker: Worker) {
    client_logger::init_logger();

    info!("graphics initialisation");
}

pub fn entry2() {
    client_logger::init_logger();

    info!("begin running wasm application");

    run_application()
}

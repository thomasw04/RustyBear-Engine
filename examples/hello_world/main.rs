#![allow(non_snake_case)]

use std::path::Path;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use RustyBear_Engine::{
    context::Context, environment::config::Config, logging, window::Window, RustyRuntime,
};

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
fn main() {
    logging::init();
    println!();

    //Create the config and init the example project.
    let mut config = Config::new(None);
    config.find_project(Path::new("examples/hello_world")).unwrap();

    //Create the window from the config and create the context.
    let mut window = Window::new("{}".to_string());
    window.native.set_ime_allowed(true);
    window.native.set_cursor_visible(false);

    let context = pollster::block_on(Context::new(window.native.clone(), config));

    //Create and init the application
    let myapp = RustyRuntime::new(&context);

    //Move my app and window into the context. And run the app.
    context.run(myapp, window);
}

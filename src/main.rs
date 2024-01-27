#![allow(non_snake_case)]

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use RustyBear_Engine::{
    context::Context, environment::config::Config, logging, window::Window, RustyRuntime,
};

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
fn main() {
    logging::init();
    println!();

    let config = Config::new(None);

    //Create the window from the config and create the context.
    let window = Window::new("{}".to_string());
    let context = pollster::block_on(Context::new(window.native.clone(), config));

    //Create and init the application
    let myapp = RustyRuntime::new(&context);

    //Move my app and window into the context. And run the app.
    context.run(myapp, window);
}

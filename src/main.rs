#![allow(non_snake_case)]

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
fn main() {
    RustyBear_Engine::example_app();
}

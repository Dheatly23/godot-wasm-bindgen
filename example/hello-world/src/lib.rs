use godot_wasm_bindgen::prelude::*;

#[godot_wasm_bindgen]
pub fn adder(a: i32, b: i32) -> i32 {
    a + b
}

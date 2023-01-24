# godot-wasm-bindgen
Experimental bindgen for [godot-wasm](https://github.com/Dheatly23/godot-wasm).

## ⚠ !!!WARNING EXPERIMENTAL!!! ⚠

This entire repository is still experimental, even more than `godot-wasm`.
It uses the experimental Godot API hooks to directly manipulate Godot objects.
Many things will break future compatibility, including:

- Format of internal data.
- Structure of the macro
- Publicly exposed objects and API.
- What version of `godot-wasm` is compatible.
- Etc...

## Introduction

This repository contains the crate for manipulating Godot objects.
It also creates shim to help export/imported function converts from and to Godot objects.

## How to Use

It models (and borrow ideas) from `wasm-bindgen`.
For the most part, use the `godot_wasm_bindgen` macro to encapsulate your exports/imports.
There is also an opaque `GodotValue` type that wraps a Godot object.

To actually generates the shim/bindgen, use the `godot-wasm-bindgen-cli` crate.

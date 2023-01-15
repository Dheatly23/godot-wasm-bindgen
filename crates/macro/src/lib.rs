mod parser;
mod util;

extern crate proc_macro;
#[macro_use]
extern crate syn;

use proc_macro::TokenStream;
use quote::ToTokens;

#[proc_macro_attribute]
pub fn godot_wasm_bindgen(metadata: TokenStream, input: TokenStream) -> TokenStream {
    let metadata = parse_macro_input!(metadata as parser::BindgenMetadata);
    let item = match parser::BindgenInput::process(metadata, input.into()) {
        Ok(v) => v,
        Err(e) => {
            return e.into_compile_error().into();
        }
    };

    let ret = item.into_token_stream();
    #[cfg(feature = "xxx_debug_print_generated_code")]
    println!("{}", ret);
    ret.into()
}

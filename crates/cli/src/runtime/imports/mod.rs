mod array;
mod other;
mod pool_array;
mod primitive;
mod string;
mod string_array;
mod typeis;

use std::collections::HashMap;

use anyhow::Error;
use walrus::{FunctionId, ImportKind, Module};

use crate::runtime::RuntimeData;
use crate::util::map_substitute_funcs;

const MODULE_NAME: &str = "godot_wasm";
const EXTERNREF_MODULE: &str = "godot_object_v2";

macro_rules! imports {
    (($mod:expr, $func_map:expr, $runtime:expr) => [$($f:ident),* $(,)?]) => {$(
        $f::generate_imports(&mut *$mod, &mut $func_map, &*$runtime)?
    );*};
}

pub fn generate_imports(module: &mut Module, runtime: &RuntimeData) -> Result<(), Error> {
    let mut func_map = HashMap::new();

    imports!((module, func_map, runtime) => [
        array,
        other,
        pool_array,
        primitive,
        string,
        string_array,
        typeis,
    ]);

    println!("{:?}", func_map);

    map_substitute_funcs(module, &func_map);

    Ok(())
}

fn replace_import<F>(
    module: &mut Module,
    func_map: &mut HashMap<FunctionId, FunctionId>,
    name: &str,
    f: F,
) -> Result<(), Error>
where
    for<'a> F: FnOnce(&'a mut Module) -> Result<FunctionId, Error>,
{
    let import_id = match module.imports.find(MODULE_NAME, name) {
        Some(v) => v,
        None => return Ok(()),
    };

    if let ImportKind::Function(source_id) = module.imports.get(import_id).kind {
        module.imports.delete(import_id);
        func_map.insert(source_id, f(&mut *module)?);
    }

    Ok(())
}

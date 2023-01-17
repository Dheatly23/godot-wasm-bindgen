mod array;
mod other;
mod pool_array;
mod primitive;
mod string;
mod string_array;
mod typeis;

use std::collections::HashMap;

use anyhow::Error;
use walrus::ir::{dfs_pre_order_mut, VisitorMut};
use walrus::{FunctionId, GlobalKind, ImportKind, InitExpr, Module};

use crate::runtime::RuntimeData;

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

    struct Substitutor<'a>(&'a HashMap<FunctionId, FunctionId>);

    impl VisitorMut for Substitutor<'_> {
        fn visit_call_mut(&mut self, instr: &mut walrus::ir::Call) {
            if let Some(&id) = self.0.get(&instr.func) {
                println!("{:?}", instr);
                instr.func = id;
            }
        }
    }

    {
        let mut substitutor = Substitutor(&func_map);

        for (_, f) in module.funcs.iter_local_mut() {
            dfs_pre_order_mut(&mut substitutor, f, f.entry_block());
        }
    }

    for e in module.elements.iter_mut() {
        if !matches!(e.ty, walrus::ValType::Funcref) {
            continue;
        }

        for i in e.members.iter_mut().filter_map(|i| i.as_mut()) {
            if let Some(&id) = func_map.get(&i) {
                *i = id;
            }
        }
    }

    {
        let ids: Vec<_> = module
            .globals
            .iter()
            .filter_map(|g| match &g.kind {
                GlobalKind::Local(InitExpr::RefFunc(_)) => Some(g.id()),
                _ => None,
            })
            .collect();

        for id in ids {
            if let GlobalKind::Local(InitExpr::RefFunc(i)) = &mut module.globals.get_mut(id).kind {
                if let Some(&id) = func_map.get(&i) {
                    *i = id;
                }
            }
        }
    }

    for id in func_map.into_keys() {
        module.funcs.delete(id);
    }

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

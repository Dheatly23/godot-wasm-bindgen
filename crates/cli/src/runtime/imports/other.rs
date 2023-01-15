use std::collections::HashMap;

use anyhow::Error;
use walrus::{FunctionBuilder, FunctionId, Module, ValType};

use super::replace_import;
use crate::runtime::RuntimeData;

pub fn generate_imports(
    module: &mut Module,
    func_map: &mut HashMap<FunctionId, FunctionId>,
    runtime: &RuntimeData,
) -> Result<(), Error> {
    replace_import(&mut *module, &mut *func_map, "duplicate", |module| {
        let mut builder = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);

        builder.name(String::from("godot_wasm.duplicate"));

        let RuntimeData {
            alloc_func,
            get_func,
            ..
        } = *runtime;

        let i = module.locals.add(ValType::I32);

        builder
            .func_body()
            .local_get(i)
            .call(get_func)
            .call(alloc_func);

        Ok(builder.finish(vec![i], &mut module.funcs))
    })?;

    replace_import(&mut *module, &mut *func_map, "delete", |module| {
        let mut builder = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[]);

        builder.name(String::from("godot_wasm.delete"));

        let RuntimeData { free_func, .. } = *runtime;

        let i = module.locals.add(ValType::I32);

        builder.func_body().local_get(i).call(free_func);

        Ok(builder.finish(vec![i], &mut module.funcs))
    })?;

    Ok(())
}

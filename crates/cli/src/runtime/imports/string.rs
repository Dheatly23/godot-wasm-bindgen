use std::collections::HashMap;

use anyhow::Error;
use walrus::{FunctionBuilder, FunctionId, Module, ValType};

use super::{replace_import, EXTERNREF_MODULE};
use crate::runtime::RuntimeData;

pub fn generate_imports(
    module: &mut Module,
    func_map: &mut HashMap<FunctionId, FunctionId>,
    runtime: &RuntimeData,
) -> Result<(), Error> {
    replace_import(&mut *module, &mut *func_map, "string.len", |module| {
        let ty = module.types.add(&[ValType::Externref], &[ValType::I32]);
        let (import_func, _) = module.add_import_func(EXTERNREF_MODULE, "string.len", ty);
        let mut builder = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);

        builder.name(String::from("godot_wasm.string.len"));

        let RuntimeData { get_func, .. } = *runtime;

        let i = module.locals.add(ValType::I32);

        builder
            .func_body()
            .local_get(i)
            .call(get_func)
            .call(import_func);

        Ok(builder.finish(vec![i], &mut module.funcs))
    })?;

    replace_import(&mut *module, &mut *func_map, "string.read", |module| {
        let ty = module
            .types
            .add(&[ValType::Externref, ValType::I32], &[ValType::I32]);
        let (import_func, _) = module.add_import_func(EXTERNREF_MODULE, "string.read", ty);
        let mut builder = FunctionBuilder::new(
            &mut module.types,
            &[ValType::I32, ValType::I32],
            &[ValType::I32],
        );

        builder.name(String::from("godot_wasm.string.read"));

        let RuntimeData { get_func, .. } = *runtime;

        let i = module.locals.add(ValType::I32);
        let p = module.locals.add(ValType::I32);

        builder
            .func_body()
            .local_get(i)
            .call(get_func)
            .local_get(p)
            .call(import_func);

        Ok(builder.finish(vec![i, p], &mut module.funcs))
    })?;

    replace_import(&mut *module, &mut *func_map, "string.write", |module| {
        let ty = module
            .types
            .add(&[ValType::I32, ValType::I32], &[ValType::Externref]);
        let (import_func, _) = module.add_import_func(EXTERNREF_MODULE, "string.write", ty);
        let mut builder = FunctionBuilder::new(
            &mut module.types,
            &[ValType::I32, ValType::I32],
            &[ValType::I32],
        );

        builder.name(String::from("godot_wasm.string.write"));

        let RuntimeData { alloc_func, .. } = *runtime;

        let p = module.locals.add(ValType::I32);
        let n = module.locals.add(ValType::I32);

        builder
            .func_body()
            .local_get(p)
            .local_get(n)
            .call(import_func)
            .call(alloc_func);

        Ok(builder.finish(vec![p, n], &mut module.funcs))
    })?;

    Ok(())
}

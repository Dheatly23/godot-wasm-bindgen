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
    replace_import(&mut *module, &mut *func_map, "array.new", |module| {
        let ty = module.types.add(&[], &[ValType::Externref]);
        let (import_func, _) = module.add_import_func(EXTERNREF_MODULE, "array.new", ty);
        let mut builder = FunctionBuilder::new(&mut module.types, &[], &[ValType::I32]);

        builder.name(String::from("godot_wasm.array.new"));

        let RuntimeData { alloc_func, .. } = *runtime;

        builder.func_body().call(import_func).call(alloc_func);

        Ok(builder.finish(vec![], &mut module.funcs))
    })?;

    replace_import(&mut *module, &mut *func_map, "array.len", |module| {
        let ty = module.types.add(&[ValType::Externref], &[ValType::I32]);
        let (import_func, _) = module.add_import_func(EXTERNREF_MODULE, "array.len", ty);
        let mut builder = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);

        builder.name(String::from("godot_wasm.array.len"));

        let RuntimeData { get_func, .. } = *runtime;

        let i = module.locals.add(ValType::I32);

        builder
            .func_body()
            .local_get(i)
            .call(get_func)
            .call(import_func);

        Ok(builder.finish(vec![i], &mut module.funcs))
    })?;

    replace_import(&mut *module, &mut *func_map, "array.get", |module| {
        let ty = module
            .types
            .add(&[ValType::Externref, ValType::I32], &[ValType::Externref]);
        let (import_func, _) = module.add_import_func(EXTERNREF_MODULE, "array.get", ty);
        let mut builder = FunctionBuilder::new(
            &mut module.types,
            &[ValType::I32, ValType::I32],
            &[ValType::I32],
        );

        builder.name(String::from("godot_wasm.array.get"));

        let RuntimeData {
            get_func,
            alloc_func,
            ..
        } = *runtime;

        let o = module.locals.add(ValType::I32);
        let i = module.locals.add(ValType::I32);

        builder
            .func_body()
            .local_get(o)
            .call(get_func)
            .local_get(i)
            .call(import_func)
            .call(alloc_func);

        Ok(builder.finish(vec![o, i], &mut module.funcs))
    })?;

    replace_import(&mut *module, &mut *func_map, "array.set", |module| {
        let ty = module
            .types
            .add(&[ValType::Externref, ValType::I32, ValType::Externref], &[]);
        let (import_func, _) = module.add_import_func(EXTERNREF_MODULE, "array.set", ty);
        let mut builder = FunctionBuilder::new(
            &mut module.types,
            &[ValType::I32, ValType::I32, ValType::I32],
            &[],
        );

        builder.name(String::from("godot_wasm.array.set"));

        let RuntimeData { get_func, .. } = *runtime;

        let o = module.locals.add(ValType::I32);
        let i = module.locals.add(ValType::I32);
        let v = module.locals.add(ValType::I32);

        builder
            .func_body()
            .local_get(o)
            .call(get_func)
            .local_get(i)
            .local_get(v)
            .call(get_func)
            .call(import_func);

        Ok(builder.finish(vec![o, i, v], &mut module.funcs))
    })?;

    replace_import(&mut *module, &mut *func_map, "array.count", |module| {
        let ty = module
            .types
            .add(&[ValType::Externref, ValType::Externref], &[ValType::I32]);
        let (import_func, _) = module.add_import_func(EXTERNREF_MODULE, "array.count", ty);
        let mut builder = FunctionBuilder::new(
            &mut module.types,
            &[ValType::I32, ValType::I32],
            &[ValType::I32],
        );

        builder.name(String::from("godot_wasm.array.count"));

        let RuntimeData { get_func, .. } = *runtime;

        let o = module.locals.add(ValType::I32);
        let v = module.locals.add(ValType::I32);

        builder
            .func_body()
            .local_get(o)
            .call(get_func)
            .local_get(v)
            .call(get_func)
            .call(import_func);

        Ok(builder.finish(vec![o, v], &mut module.funcs))
    })?;

    replace_import(&mut *module, &mut *func_map, "array.contains", |module| {
        let ty = module
            .types
            .add(&[ValType::Externref, ValType::Externref], &[ValType::I32]);
        let (import_func, _) = module.add_import_func(EXTERNREF_MODULE, "array.contains", ty);
        let mut builder = FunctionBuilder::new(
            &mut module.types,
            &[ValType::I32, ValType::I32],
            &[ValType::I32],
        );

        builder.name(String::from("godot_wasm.array.contains"));

        let RuntimeData { get_func, .. } = *runtime;

        let o = module.locals.add(ValType::I32);
        let v = module.locals.add(ValType::I32);

        builder
            .func_body()
            .local_get(o)
            .call(get_func)
            .local_get(v)
            .call(get_func)
            .call(import_func);

        Ok(builder.finish(vec![o, v], &mut module.funcs))
    })?;

    replace_import(&mut *module, &mut *func_map, "array.find", |module| {
        let ty = module.types.add(
            &[ValType::Externref, ValType::Externref, ValType::I32],
            &[ValType::I32],
        );
        let (import_func, _) = module.add_import_func(EXTERNREF_MODULE, "array.find", ty);
        let mut builder = FunctionBuilder::new(
            &mut module.types,
            &[ValType::I32, ValType::I32, ValType::I32],
            &[ValType::I32],
        );

        builder.name(String::from("godot_wasm.array.find"));

        let RuntimeData { get_func, .. } = *runtime;

        let o = module.locals.add(ValType::I32);
        let v = module.locals.add(ValType::I32);
        let f = module.locals.add(ValType::I32);

        builder
            .func_body()
            .local_get(o)
            .call(get_func)
            .local_get(v)
            .call(get_func)
            .local_get(f)
            .call(import_func);

        Ok(builder.finish(vec![o, v, f], &mut module.funcs))
    })?;

    replace_import(&mut *module, &mut *func_map, "array.rfind", |module| {
        let ty = module.types.add(
            &[ValType::Externref, ValType::Externref, ValType::I32],
            &[ValType::I32],
        );
        let (import_func, _) = module.add_import_func(EXTERNREF_MODULE, "array.rfind", ty);
        let mut builder = FunctionBuilder::new(
            &mut module.types,
            &[ValType::I32, ValType::I32, ValType::I32],
            &[ValType::I32],
        );

        builder.name(String::from("godot_wasm.array.rfind"));

        let RuntimeData { get_func, .. } = *runtime;

        let o = module.locals.add(ValType::I32);
        let v = module.locals.add(ValType::I32);
        let f = module.locals.add(ValType::I32);

        builder
            .func_body()
            .local_get(o)
            .call(get_func)
            .local_get(v)
            .call(get_func)
            .local_get(f)
            .call(import_func);

        Ok(builder.finish(vec![o, v, f], &mut module.funcs))
    })?;

    replace_import(&mut *module, &mut *func_map, "array.invert", |module| {
        let ty = module.types.add(&[ValType::Externref], &[]);
        let (import_func, _) = module.add_import_func(EXTERNREF_MODULE, "array.invert", ty);
        let mut builder = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[]);

        builder.name(String::from("godot_wasm.array.invert"));

        let RuntimeData { get_func, .. } = *runtime;

        let i = module.locals.add(ValType::I32);

        builder
            .func_body()
            .local_get(i)
            .call(get_func)
            .call(import_func);

        Ok(builder.finish(vec![i], &mut module.funcs))
    })?;

    replace_import(&mut *module, &mut *func_map, "array.sort", |module| {
        let ty = module.types.add(&[ValType::Externref], &[]);
        let (import_func, _) = module.add_import_func(EXTERNREF_MODULE, "array.sort", ty);
        let mut builder = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[]);

        builder.name(String::from("godot_wasm.array.sort"));

        let RuntimeData { get_func, .. } = *runtime;

        let i = module.locals.add(ValType::I32);

        builder
            .func_body()
            .local_get(i)
            .call(get_func)
            .call(import_func);

        Ok(builder.finish(vec![i], &mut module.funcs))
    })?;

    replace_import(&mut *module, &mut *func_map, "array.clear", |module| {
        let ty = module.types.add(&[ValType::Externref], &[]);
        let (import_func, _) = module.add_import_func(EXTERNREF_MODULE, "array.clear", ty);
        let mut builder = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[]);

        builder.name(String::from("godot_wasm.array.clear"));

        let RuntimeData { get_func, .. } = *runtime;

        let i = module.locals.add(ValType::I32);

        builder
            .func_body()
            .local_get(i)
            .call(get_func)
            .call(import_func);

        Ok(builder.finish(vec![i], &mut module.funcs))
    })?;

    replace_import(&mut *module, &mut *func_map, "array.duplicate", |module| {
        let ty = module
            .types
            .add(&[ValType::Externref], &[ValType::Externref]);
        let (import_func, _) = module.add_import_func(EXTERNREF_MODULE, "array.duplicate", ty);
        let mut builder = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);

        builder.name(String::from("godot_wasm.array.duplicate"));

        let RuntimeData {
            get_func,
            alloc_func,
            ..
        } = *runtime;

        let i = module.locals.add(ValType::I32);

        builder
            .func_body()
            .local_get(i)
            .call(get_func)
            .call(import_func)
            .call(alloc_func);

        Ok(builder.finish(vec![i], &mut module.funcs))
    })?;

    Ok(())
}

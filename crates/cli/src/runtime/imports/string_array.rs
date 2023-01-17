use std::collections::HashMap;

use anyhow::Error;
use walrus::ir::{BinaryOp, LoadKind, MemArg, StoreKind, Value};
use walrus::{FunctionBuilder, FunctionId, Module, ValType};

use super::{replace_import, EXTERNREF_MODULE};
use crate::runtime::RuntimeData;

pub fn generate_imports(
    module: &mut Module,
    func_map: &mut HashMap<FunctionId, FunctionId>,
    runtime: &RuntimeData,
) -> Result<(), Error> {
    replace_import(&mut *module, &mut *func_map, "string_array.len", |module| {
        let ty = module.types.add(&[ValType::Externref], &[ValType::I32]);
        let (import_func, _) = module.add_import_func(EXTERNREF_MODULE, "string_array.len", ty);
        let mut builder = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);

        builder.name(String::from("godot_wasm.string_array.len"));

        let RuntimeData { get_func, .. } = *runtime;

        let i = module.locals.add(ValType::I32);

        builder
            .func_body()
            .local_get(i)
            .call(get_func)
            .call(import_func);

        Ok(builder.finish(vec![i], &mut module.funcs))
    })?;

    replace_import(&mut *module, &mut *func_map, "string_array.get", |module| {
        let ty = module
            .types
            .add(&[ValType::Externref, ValType::I32], &[ValType::Externref]);
        let (import_func, _) = module.add_import_func(EXTERNREF_MODULE, "string_array.get", ty);
        let mut builder = FunctionBuilder::new(
            &mut module.types,
            &[ValType::I32, ValType::I32],
            &[ValType::I32],
        );

        builder.name(String::from("godot_wasm.string.read"));

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

    replace_import(
        &mut *module,
        &mut *func_map,
        "string_array.get_many",
        |module| {
            let ty = module.types.add(
                &[ValType::Externref, ValType::I32, ValType::Funcref],
                &[ValType::I32],
            );
            let (import_func, _) =
                module.add_import_func(EXTERNREF_MODULE, "string_array.get_many", ty);

            let RuntimeData {
                main_memory,
                declare_funcs,
                index_global,
                limit_global,
                get_func,
                alloc_func,
                ..
            } = *runtime;

            let mut builder =
                FunctionBuilder::new(&mut module.types, &[ValType::Externref], &[ValType::I32]);

            builder.name(String::from("godot_wasm.string_array.get_many.helper"));

            let o = module.locals.add(ValType::Externref);
            let i = module.locals.add(ValType::I32);

            builder
                .func_body()
                .global_get(index_global)
                .local_tee(i)
                .local_get(o)
                .call(alloc_func)
                .store(
                    main_memory,
                    StoreKind::I32 { atomic: false },
                    MemArg {
                        align: 2,
                        offset: 0,
                    },
                )
                .local_get(i)
                .const_(Value::I32(4))
                .binop(BinaryOp::I32Add)
                .local_tee(i)
                .global_set(index_global)
                .local_get(i)
                .global_get(limit_global)
                .binop(BinaryOp::I32Ne);

            let helper_func = builder.finish(vec![o, i], &mut module.funcs);
            module
                .elements
                .get_mut(declare_funcs)
                .members
                .push(Some(helper_func));

            builder = FunctionBuilder::new(
                &mut module.types,
                &[ValType::I32, ValType::I32, ValType::I32, ValType::I32],
                &[ValType::I32],
            );

            builder.name(String::from("godot_wasm.string_array.get_many"));

            let o = module.locals.add(ValType::I32);
            let i = module.locals.add(ValType::I32);
            let s = module.locals.add(ValType::I32);
            let e = module.locals.add(ValType::I32);

            builder
                .func_body()
                .local_get(o)
                .call(get_func)
                .local_get(i)
                .ref_func(helper_func)
                .local_get(s)
                .global_set(index_global)
                .local_get(e)
                .global_set(limit_global)
                .call(import_func);

            Ok(builder.finish(vec![o, i, s, e], &mut module.funcs))
        },
    )?;

    replace_import(
        &mut *module,
        &mut *func_map,
        "string_array.build",
        |module| {
            let ty = module.types.add(&[ValType::Funcref], &[ValType::Externref]);
            let (import_func, _) =
                module.add_import_func(EXTERNREF_MODULE, "string_array.build", ty);

            let RuntimeData {
                main_memory,
                declare_funcs,
                index_global,
                limit_global,
                get_func,
                alloc_func,
                ..
            } = *runtime;

            let mut builder = FunctionBuilder::new(
                &mut module.types,
                &[ValType::I32],
                &[ValType::Externref, ValType::I32],
            );

            builder.name(String::from("godot_wasm.string_array.build.helper"));

            let i = module.locals.add(ValType::I32);

            builder
                .func_body()
                .local_get(i)
                .const_(Value::I32(4))
                .binop(BinaryOp::I32Mul)
                .global_get(index_global)
                .binop(BinaryOp::I32Add)
                .local_tee(i)
                .load(
                    main_memory,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 2,
                        offset: 0,
                    },
                )
                .call(get_func)
                .local_get(i)
                .global_get(limit_global)
                .binop(BinaryOp::I32Ne);

            let helper_func = builder.finish(vec![i], &mut module.funcs);
            module
                .elements
                .get_mut(declare_funcs)
                .members
                .push(Some(helper_func));

            builder = FunctionBuilder::new(
                &mut module.types,
                &[ValType::I32, ValType::I32],
                &[ValType::I32],
            );

            builder.name(String::from("godot_wasm.string_array.build"));

            let s = module.locals.add(ValType::I32);
            let e = module.locals.add(ValType::I32);

            builder
                .func_body()
                .ref_func(helper_func)
                .local_get(s)
                .global_set(index_global)
                .local_get(e)
                .global_set(limit_global)
                .call(import_func)
                .call(alloc_func);

            Ok(builder.finish(vec![s, e], &mut module.funcs))
        },
    )?;

    Ok(())
}

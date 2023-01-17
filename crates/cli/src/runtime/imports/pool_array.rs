use std::collections::HashMap;

use anyhow::Error;
use walrus::{FunctionBuilder, FunctionId, Module, ValType};

use super::{replace_import, EXTERNREF_MODULE};
use crate::runtime::RuntimeData;

macro_rules! generate {
    ($(($lname:literal, $rname:literal, $wname:literal)),* $(,)?) => {
        pub fn generate_imports(
            module: &mut Module,
            func_map: &mut HashMap<FunctionId, FunctionId>,
            runtime: &RuntimeData
        ) -> Result<(), Error> {
            $(
                replace_import(&mut *module, &mut *func_map, $lname, |module| {
                    let ty = module.types.add(
                        &[ValType::Externref],
                        &[ValType::I32],
                    );
                    let (import_func, _) = module.add_import_func(
                        EXTERNREF_MODULE,
                        $rname,
                        ty,
                    );
                    let mut builder = FunctionBuilder::new(
                        &mut module.types,
                        &[ValType::I32],
                        &[ValType::I32],
                    );

                    builder.name(String::from(concat!("godot_wasm.", $rname)));

                    let RuntimeData {
                        get_func,
                        ..
                    } = *runtime;

                    let i = module.locals.add(ValType::I32);

                    builder
                        .func_body()
                        .local_get(i)
                        .call(get_func)
                        .call(import_func);

                    Ok(builder.finish(vec![i], &mut module.funcs))
                })?;

                replace_import(&mut *module, &mut *func_map, $rname, |module| {
                    let ty = module.types.add(
                        &[ValType::Externref, ValType::I32],
                        &[ValType::I32],
                    );
                    let (import_func, _) = module.add_import_func(
                        EXTERNREF_MODULE,
                        $rname,
                        ty,
                    );
                    let mut builder = FunctionBuilder::new(
                        &mut module.types,
                        &[ValType::I32, ValType::I32],
                        &[ValType::I32],
                    );

                    builder.name(String::from(concat!("godot_wasm.", $rname)));

                    let RuntimeData {
                        get_func,
                        ..
                    } = *runtime;

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

                replace_import(&mut *module, &mut *func_map, $wname, |module| {
                    let ty = module.types.add(
                        &[ValType::I32],
                        &[ValType::Externref],
                    );
                    let (import_func, _) = module.add_import_func(
                        EXTERNREF_MODULE,
                        $wname,
                        ty,
                    );
                    let mut builder = FunctionBuilder::new(
                        &mut module.types,
                        &[ValType::I32],
                        &[ValType::I32],
                    );

                    builder.name(String::from(concat!("godot_wasm.", $wname)));

                    let RuntimeData {
                        alloc_func,
                        ..
                    } = *runtime;

                    let p = module.locals.add(ValType::I32);

                    builder
                        .func_body()
                        .local_get(p)
                        .call(import_func)
                        .call(alloc_func);

                    Ok(builder.finish(vec![p], &mut module.funcs))
                })?;
            )*

            Ok(())
        }
    };
}

generate!(
    ("byte_array.len", "byte_array.read", "byte_array.write"),
    ("int_array.len", "int_array.read", "int_array.write"),
    ("float_array.len", "float_array.read", "float_array.write"),
    (
        "vector2_array.len",
        "vector2_array.read",
        "vector2_array.write"
    ),
    (
        "vector3_array.len",
        "vector3_array.read",
        "vector3_array.write"
    ),
    ("color_array.len", "color_array.read", "color_array.write"),
);

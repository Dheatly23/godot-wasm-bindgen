use std::collections::HashMap;

use anyhow::Error;
use walrus::{FunctionBuilder, FunctionId, Module, ValType};

use super::{replace_import, MODULE_NAME};
use crate::runtime::RuntimeData;

macro_rules! generate {
    ($(($rname:literal, $wname:literal)),* $(,)?) => {
        pub fn generate_imports(
            module: &mut Module,
            func_map: &mut HashMap<FunctionId, FunctionId>,
            runtime: &RuntimeData
        ) -> Result<(), Error> {
            $(
                replace_import(&mut *module, &mut *func_map, $rname, |module| {
                    let ty = module.types.add(
                        &[ValType::Externref, ValType::I32],
                        &[ValType::I32],
                    );
                    let (import_func, _) = module.add_import_func(
                        MODULE_NAME,
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
                        MODULE_NAME,
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
    ("bool.read", "bool.write"),
    ("int.read", "int.write"),
    ("float.read", "float.write"),
    ("vector2.read", "vector2.write"),
    ("vector3.read", "vector3.write"),
    ("rect2.read", "rect2.write"),
    ("transform2d.read", "transform2d.write"),
    ("plane.read", "plane.write"),
    ("quat.read", "quat.write"),
    ("aabb.read", "aabb.write"),
    ("basis.read", "basis.write"),
    ("transform.read", "transform.write"),
    ("color.read", "color.write"),
);

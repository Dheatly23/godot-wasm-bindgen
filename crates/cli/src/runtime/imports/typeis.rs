use std::collections::HashMap;

use anyhow::Error;
use walrus::{FunctionBuilder, FunctionId, Module, ValType};

use super::{replace_import, MODULE_NAME};
use crate::runtime::RuntimeData;

macro_rules! generate {
    ($($iname:literal),* $(,)?) => {
        pub fn generate_imports(
            module: &mut Module,
            func_map: &mut HashMap<FunctionId, FunctionId>,
            runtime: &RuntimeData
        ) -> Result<(), Error> {
            $(
                replace_import(&mut *module, &mut *func_map, $iname, |module| {
                    let ty = module.types.add(
                        &[ValType::Externref],
                        &[ValType::I32],
                    );
                    let (import_func, _) = module.add_import_func(
                        MODULE_NAME,
                        $iname,
                        ty,
                    );
                    let mut builder = FunctionBuilder::new(
                        &mut module.types,
                        &[ValType::I32],
                        &[ValType::I32],
                    );

                    builder.name(String::from(concat!("godot_wasm.", $iname)));

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
            )*

            Ok(())
        }
    };
}

generate!(
    "bool.is",
    "int.is",
    "float.is",
    "string.is",
    "vector2.is",
    "rect2.is",
    "vector3.is",
    "transform2d.is",
    "plane.is",
    "quat.is",
    "aabb.is",
    "basis.is",
    "transform.is",
    "color.is",
    "nodepath.is",
    "rid.is",
    "object.is",
    "dictionary.is",
    "array.is",
    "byte_array.is",
    "int_array.is",
    "float_array.is",
    "string_array.is",
    "vector2_array.is",
    "vector3_array.is",
    "color_array.is",
);

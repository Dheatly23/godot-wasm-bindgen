use std::collections::HashMap;

use anyhow::{bail, Error};
use walrus::ir::{BinaryOp, UnaryOp, Value};
use walrus::{ExportItem, FunctionBuilder, ImportKind, Module, ValType};

use crate::decode::{
    ArgType, ExportFunction, FunctionArgs, GodotWasmBindgenData, ImportFunction, SymbolType,
};
use crate::runtime::RuntimeData;
use crate::util::map_substitute_funcs;

pub fn substitute_exports(
    module: &mut Module,
    custom_data: &GodotWasmBindgenData,
    runtime: &RuntimeData,
) -> Result<(), Error> {
    let RuntimeData {
        alloc_func,
        free_func,
        get_func,
        ..
    } = *runtime;

    let exports: HashMap<_, _> = custom_data
        .symbols
        .iter()
        .filter_map(|symbol| match &symbol.inner {
            SymbolType::ExportFunction(e) => Some((&e.name as &str, &*e)),
            _ => None,
        })
        .collect();

    for e in module.exports.iter_mut() {
        let f = match &mut e.item {
            ExportItem::Function(f) => f,
            _ => continue,
        };

        let ExportFunction {
            args: FunctionArgs { params, results },
            ..
        } = match exports.get(&e.name as &str) {
            Some(&v) => v,
            None => continue,
        };

        let func_params: Vec<_> = params.iter().copied().map(ValType::from).collect();
        let func_results: Vec<_> = results.iter().copied().map(ValType::from).collect();

        {
            let ty = module.types.get(module.funcs.get(*f).ty());
            let (params, results) = (ty.params(), ty.results());

            if func_params.len() != params.len() {
                bail!(
                    "Parameter length mismatch! ({} != {})",
                    func_params.len(),
                    params.len()
                );
            }
            if func_results.len() != results.len() {
                bail!(
                    "Parameter length mismatch! ({} != {})",
                    func_results.len(),
                    results.len()
                );
            }

            for (i, (&a, &b)) in func_params.iter().zip(params.iter()).enumerate() {
                let a = match a {
                    ValType::Externref => ValType::I32,
                    v => v,
                };
                if a != b {
                    bail!("Parameter type mismatch at {i} ({a} != {b})");
                }
            }
            for (i, (&a, &b)) in func_results.iter().zip(results.iter()).enumerate() {
                let a = match a {
                    ValType::Externref => ValType::I32,
                    v => v,
                };
                if a != b {
                    bail!("Result type mismatch at {i} ({a} != {b})");
                }
            }
        }

        let mut builder = FunctionBuilder::new(&mut module.types, &func_params, &func_results);

        builder.name(e.name.clone());

        let var_params: Vec<_> = func_params
            .iter()
            .map(|&ty| module.locals.add(ty))
            .collect();
        let var_results: Vec<_> = func_results
            .iter()
            .map(|&ty| module.locals.add(ty))
            .collect();
        let temp = module.locals.add(ValType::I32);

        let mut body = builder.func_body();

        for (i, &p) in var_params.iter().enumerate() {
            body.local_get(p);
            match params[i] {
                ArgType::U8 => body.const_(Value::I32(255)).binop(BinaryOp::I32And),
                ArgType::I8 => body.unop(UnaryOp::I32Extend8S),
                ArgType::U16 => body.const_(Value::I32(65535)).binop(BinaryOp::I32And),
                ArgType::I16 => body.unop(UnaryOp::I32Extend16S),
                ArgType::U32
                | ArgType::I32
                | ArgType::U64
                | ArgType::I64
                | ArgType::F32
                | ArgType::F64 => &mut body,
                ArgType::GodotValue => body.call(alloc_func),
            };
        }

        body.call(*f);

        for (i, &r) in var_results.iter().enumerate().rev() {
            match results[i] {
                ArgType::GodotValue => body
                    .local_tee(temp)
                    .call(get_func)
                    .local_set(r)
                    .local_get(temp)
                    .call(free_func),
                _ => body.local_set(r),
            };
        }

        for &r in &var_results {
            body.local_get(r);
        }

        *f = builder.finish(var_params, &mut module.funcs);
    }

    Ok(())
}

pub fn substitute_imports(
    module: &mut Module,
    custom_data: &GodotWasmBindgenData,
    runtime: &RuntimeData,
) -> Result<(), Error> {
    let RuntimeData {
        alloc_func,
        free_func,
        get_func,
        ..
    } = *runtime;

    let mut func_map = HashMap::new();

    let imports: HashMap<_, _> = custom_data
        .symbols
        .iter()
        .filter_map(|symbol| match &symbol.inner {
            SymbolType::ImportFunction(e) => Some(((&e.module as &str, &e.name as &str), &*e)),
            _ => None,
        })
        .collect();

    for i in module.imports.iter_mut() {
        let id = i.id();
        let f = match &mut i.kind {
            ImportKind::Function(f) => f,
            _ => continue,
        };

        let ImportFunction {
            module: module_,
            name,
            args: FunctionArgs { params, results },
        } = match imports.get(&(&i.module as &str, &i.name as &str)) {
            Some(&v) => v,
            None => continue,
        };

        let (name, ty) = {
            let f = module.funcs.get(*f);
            (
                f.name
                    .clone()
                    .unwrap_or_else(|| format!("{module_}.{name}")),
                f.ty(),
            )
        };

        let mut func_params: Vec<_> = params.iter().copied().map(ValType::from).collect();
        let mut func_results: Vec<_> = results.iter().copied().map(ValType::from).collect();

        let mut f_ = module
            .funcs
            .add_import(module.types.add(&func_params, &func_results), id);

        for i in func_params.iter_mut().chain(func_results.iter_mut()) {
            if matches!(i, ValType::Externref) {
                *i = ValType::I32;
            }
        }

        {
            let ty = module.types.get(ty);
            let (params, results) = (ty.params(), ty.results());

            if func_params.len() != params.len() {
                bail!(
                    "Parameter length mismatch! ({} != {})",
                    func_params.len(),
                    params.len()
                );
            }
            if func_results.len() != results.len() {
                bail!(
                    "Parameter length mismatch! ({} != {})",
                    func_results.len(),
                    results.len()
                );
            }

            for (i, (&a, &b)) in func_params.iter().zip(params.iter()).enumerate() {
                if a != b {
                    bail!("Parameter type mismatch at {i} ({a} != {b})");
                }
            }
            for (i, (&a, &b)) in func_results.iter().zip(results.iter()).enumerate() {
                if a != b {
                    bail!("Result type mismatch at {i} ({a} != {b})");
                }
            }
        }

        let mut builder = FunctionBuilder::new(&mut module.types, &func_params, &func_results);

        builder.name(name);

        let var_params: Vec<_> = func_params
            .iter()
            .map(|&ty| module.locals.add(ty))
            .collect();
        let var_results: Vec<_> = func_results
            .iter()
            .map(|&ty| module.locals.add(ty))
            .collect();

        let mut body = builder.func_body();

        for (i, &p) in var_params.iter().enumerate() {
            match params[i] {
                ArgType::GodotValue => body
                    .local_get(p)
                    .call(get_func)
                    .local_get(p)
                    .call(free_func),
                _ => body.local_get(p),
            };
        }

        body.call(f_);

        for (i, &r) in var_results.iter().enumerate().rev() {
            match results[i] {
                ArgType::U8 => body.const_(Value::I32(255)).binop(BinaryOp::I32And),
                ArgType::I8 => body.unop(UnaryOp::I32Extend8S),
                ArgType::U16 => body.const_(Value::I32(65535)).binop(BinaryOp::I32And),
                ArgType::I16 => body.unop(UnaryOp::I32Extend16S),
                ArgType::U32
                | ArgType::I32
                | ArgType::U64
                | ArgType::I64
                | ArgType::F32
                | ArgType::F64 => &mut body,
                ArgType::GodotValue => body.call(alloc_func),
            }
            .local_set(r);
        }

        for &r in &var_results {
            body.local_get(r);
        }

        (*f, f_) = (f_, *f);
        module.funcs.delete(f_);
        func_map.insert(f_, builder.finish(var_params, &mut module.funcs));
    }

    map_substitute_funcs(module, &func_map);

    Ok(())
}

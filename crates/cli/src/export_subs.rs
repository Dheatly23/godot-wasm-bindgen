use std::collections::HashMap;

use anyhow::Error;
use walrus::{ExportItem, FunctionBuilder, Module, ValType};

use crate::decode::{GodotWasmBindgenData, Symbol};
use crate::runtime::RuntimeData;

pub fn substitute_exports(
    module: &mut Module,
    custom_data: GodotWasmBindgenData,
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
        .into_iter()
        .map(
            |Symbol {
                 name, extra_data, ..
             }| (name, extra_data),
        )
        .collect();

    for e in module.exports.iter_mut() {
        let f = match &mut e.item {
            ExportItem::Function(f) => f,
            _ => continue,
        };

        if !exports.contains_key(&e.name) {
            continue;
        }

        let (params, results): (Vec<_>, Vec<_>) = {
            let ty = module.types.get(module.funcs.get(*f).ty());
            let f = |_| ValType::Externref;
            (
                ty.params().iter().map(f).collect(),
                ty.results().iter().map(f).collect(),
            )
        };

        let mut builder = FunctionBuilder::new(&mut module.types, &params, &results);

        builder.name(e.name.clone());

        let params: Vec<_> = params.into_iter().map(|ty| module.locals.add(ty)).collect();
        let results: Vec<_> = results
            .into_iter()
            .map(|ty| module.locals.add(ty))
            .collect();
        let temp = module.locals.add(ValType::I32);

        let mut body = builder.func_body();

        for &p in &params {
            body.local_get(p).call(alloc_func);
        }

        body.call(*f);

        for &r in results.iter().rev() {
            body.local_tee(temp)
                .call(get_func)
                .local_set(r)
                .local_get(temp)
                .call(free_func);
        }

        for &r in &results {
            body.local_get(r);
        }

        *f = builder.finish(params, &mut module.funcs);
    }

    Ok(())
}

mod imports;

use anyhow::Error;
use walrus::ir::{BinaryOp, ExtendedLoad, LoadKind, MemArg, StoreKind, UnaryOp, Value};
use walrus::{FunctionBuilder, FunctionId, GlobalId, InitExpr, MemoryId, Module, TableId, ValType};

#[allow(dead_code)]
pub struct RuntimeData {
    extern_table: TableId,
    extern_memory: MemoryId,

    head_global: GlobalId,
    pub alloc_func: FunctionId,
    pub free_func: FunctionId,
    pub get_func: FunctionId,
}

pub fn add_runtime(module: &mut Module) -> Result<RuntimeData, Error> {
    let extern_table = module
        .tables
        .add_local(0, Some(65536), walrus::ValType::Externref);
    let extern_memory = module.memories.add_local(false, 1, None);
    let head_global = module
        .globals
        .add_local(ValType::I32, true, InitExpr::Value(Value::I32(0)));

    let alloc_func = {
        let mut builder =
            FunctionBuilder::new(&mut module.types, &[ValType::Externref], &[ValType::I32]);

        builder.name(String::from("godot_wasm.alloc"));

        let e = module.locals.add(ValType::Externref);
        let i = module.locals.add(ValType::I32);

        builder
            .func_body()
            .block(None, |b| {
                let id = b.id();
                b.local_get(e)
                    .ref_is_null()
                    .unop(UnaryOp::I32Eqz)
                    .br_if(id)
                    .const_(Value::I32(0))
                    .return_();
            })
            .global_get(head_global)
            .local_tee(i)
            .table_size(extern_table)
            .binop(BinaryOp::I32GeU)
            .if_else(
                None,
                |then| {
                    then.block(None, |b| {
                        let id = b.id();
                        b.local_get(e)
                            .const_(Value::I32(1))
                            .table_grow(extern_table)
                            .const_(Value::I32(-1))
                            .binop(BinaryOp::I32Ne)
                            .br_if(id)
                            .unreachable();
                    })
                    .const_(Value::I32(1))
                    .local_get(i)
                    .binop(BinaryOp::I32Add)
                    .global_set(head_global);
                },
                |el| {
                    el.local_get(i)
                        .const_(Value::I32(2))
                        .binop(BinaryOp::I32Mul)
                        .load(
                            extern_memory,
                            LoadKind::I32_16 {
                                kind: ExtendedLoad::ZeroExtend,
                            },
                            MemArg {
                                align: 1,
                                offset: 0,
                            },
                        )
                        .global_set(head_global);
                },
            )
            .local_get(i)
            .local_get(e)
            .table_set(extern_table)
            .local_get(i)
            .const_(Value::I32(2))
            .binop(BinaryOp::I32Mul)
            .const_(Value::I32(-1))
            .store(
                extern_memory,
                StoreKind::I32_16 { atomic: false },
                MemArg {
                    align: 1,
                    offset: 0,
                },
            )
            .local_get(i)
            .const_(Value::I32(1))
            .binop(BinaryOp::I32Add);

        builder.finish(vec![e], &mut module.funcs)
    };
    let free_func = {
        let mut builder = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[]);

        builder.name(String::from("godot_wasm.free"));

        let i = module.locals.add(ValType::I32);

        builder
            .func_body()
            .block(None, |b| {
                let id = b.id();
                b.local_get(i).br_if(id).return_();
            })
            .local_get(i)
            .const_(Value::I32(1))
            .binop(BinaryOp::I32Sub)
            .local_tee(i)
            .ref_null(ValType::Externref)
            .table_set(extern_table)
            .local_get(i)
            .const_(Value::I32(2))
            .binop(BinaryOp::I32Mul)
            .global_get(head_global)
            .store(
                extern_memory,
                StoreKind::I32_16 { atomic: false },
                MemArg {
                    align: 1,
                    offset: 0,
                },
            )
            .local_get(i)
            .global_set(head_global);

        builder.finish(vec![i], &mut module.funcs)
    };
    let get_func = {
        let mut builder =
            FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::Externref]);

        builder.name(String::from("godot_wasm.get"));

        let i = module.locals.add(ValType::I32);

        builder
            .func_body()
            .local_get(i)
            .unop(UnaryOp::I32Eqz)
            .if_else(
                Some(ValType::Externref),
                |then| {
                    then.ref_null(ValType::Externref);
                },
                |el| {
                    el.local_get(i)
                        .const_(Value::I32(1))
                        .binop(BinaryOp::I32Sub)
                        .table_get(extern_table);
                },
            );

        builder.finish(vec![i], &mut module.funcs)
    };

    let runtime = RuntimeData {
        extern_table,
        extern_memory,

        head_global,
        alloc_func,
        free_func,
        get_func,
    };

    imports::generate_imports(&mut *module, &runtime)?;

    Ok(runtime)
}

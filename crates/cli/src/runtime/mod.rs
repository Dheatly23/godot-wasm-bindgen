mod imports;

use anyhow::{bail, Error};
use walrus::ir::{BinaryOp, ExtendedLoad, LoadKind, MemArg, StoreKind, UnaryOp, Value};
use walrus::{
    ElementId, ElementKind, Export, ExportItem, FunctionBuilder, FunctionId, GlobalId, InitExpr,
    MemoryId, Module, TableId, ValType,
};

#[allow(dead_code)]
pub struct RuntimeData {
    pub main_memory: MemoryId,

    extern_table: TableId,
    extern_memory: MemoryId,
    pub declare_funcs: ElementId,
    pub index_global: GlobalId,
    pub limit_global: GlobalId,

    head_global: GlobalId,
    pub alloc_func: FunctionId,
    pub free_func: FunctionId,
    pub get_func: FunctionId,
}

pub fn add_runtime(module: &mut Module) -> Result<RuntimeData, Error> {
    let main_memory = {
        let mut it = module.exports.iter();
        loop {
            if let Some(Export { name, item, .. }) = it.next() {
                if name != "memory" {
                    continue;
                }
                if let ExportItem::Memory(mem) = item {
                    break *mem;
                }
            } else {
                bail!("No main memory exported!");
            }
        }
    };

    let extern_table = module
        .tables
        .add_local(0, Some(65536), walrus::ValType::Externref);
    let extern_memory = module.memories.add_local(false, 1, None);
    let declare_funcs = module
        .elements
        .add(ElementKind::Declared, ValType::Funcref, Vec::new());
    let index_global = module
        .globals
        .add_local(ValType::I32, true, InitExpr::Value(Value::I32(0)));
    let limit_global = module
        .globals
        .add_local(ValType::I32, true, InitExpr::Value(Value::I32(0)));
    let head_global = module
        .globals
        .add_local(ValType::I32, true, InitExpr::Value(Value::I32(0)));

    let alloc_func = {
        let mut builder =
            FunctionBuilder::new(&mut module.types, &[ValType::Externref], &[ValType::I32]);

        builder.name(String::from("godot_wasm.alloc"));

        let e = module.locals.add(ValType::Externref);
        let i = module.locals.add(ValType::I32);
        let j = module.locals.add(ValType::I32);

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
                    then.table_size(extern_table)
                        .local_set(i)
                        .block(None, |b| {
                            let id = b.id();
                            b.ref_null(ValType::Externref)
                                .const_(Value::I32(1))
                                .table_grow(extern_table)
                                .local_tee(j)
                                .const_(Value::I32(-1))
                                .binop(BinaryOp::I32Ne)
                                .br_if(id)
                                .unreachable();
                        })
                        .local_get(j)
                        .local_get(i)
                        .binop(BinaryOp::I32Sub)
                        .local_set(j)
                        .block(None, |b| {
                            let b_id = b.id();
                            b.loop_(None, |l| {
                                let l_id = l.id();
                                l.local_get(j)
                                    .unop(UnaryOp::I32Eqz)
                                    .br_if(b_id)
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
                                    .binop(BinaryOp::I32Add)
                                    .local_set(i)
                                    .local_get(j)
                                    .const_(Value::I32(1))
                                    .binop(BinaryOp::I32Sub)
                                    .local_set(j)
                                    .br(l_id);
                            });
                        })
                        .local_get(i)
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
        main_memory,
        extern_table,
        extern_memory,
        declare_funcs,
        index_global,
        limit_global,

        head_global,
        alloc_func,
        free_func,
        get_func,
    };

    imports::generate_imports(&mut *module, &runtime)?;

    Ok(runtime)
}

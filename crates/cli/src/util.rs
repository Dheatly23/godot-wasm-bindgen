#![allow(dead_code)]

use std::collections::HashMap;
use std::io::Read;
use std::iter;

use anyhow::Error;
use walrus::ir::{dfs_pre_order_mut, VisitorMut};
use walrus::{FunctionId, GlobalKind, InitExpr, Module};

pub fn tag_length(bytes: &mut Vec<u8>) {
    let mut temp = Vec::new();
    leb128::write::unsigned(&mut temp, bytes.len() as _).unwrap();
    prepend(bytes, &temp);
}

pub fn prepend(bytes: &mut Vec<u8>, data: &[u8]) {
    let l = bytes.len();
    let ld = data.len();
    bytes.extend(iter::repeat(0).take(ld));
    bytes.copy_within(0..l, ld);
    bytes[0..ld].copy_from_slice(data);
}

pub fn read_tagged<F, R>(reader: &mut dyn Read, f: F) -> Result<R, Error>
where
    for<'a> F: FnOnce(&'a mut dyn Read) -> Result<R, Error>,
{
    let len = leb128::read::unsigned(&mut *reader)?.try_into()?;
    f(&mut reader.take(len))
}

pub fn map_substitute_funcs(module: &mut Module, func_map: &HashMap<FunctionId, FunctionId>) {
    struct Substitutor<'a>(&'a HashMap<FunctionId, FunctionId>);

    impl VisitorMut for Substitutor<'_> {
        fn visit_call_mut(&mut self, instr: &mut walrus::ir::Call) {
            if let Some(&id) = self.0.get(&instr.func) {
                instr.func = id;
            }
        }
    }

    let mut substitutor = Substitutor(&func_map);

    for (_, f) in module.funcs.iter_local_mut() {
        dfs_pre_order_mut(&mut substitutor, f, f.entry_block());
    }

    for e in module.elements.iter_mut() {
        if !matches!(e.ty, walrus::ValType::Funcref) {
            continue;
        }

        for i in e.members.iter_mut().filter_map(|i| i.as_mut()) {
            if let Some(&id) = func_map.get(&i) {
                *i = id;
            }
        }
    }

    let ids: Vec<_> = module
        .globals
        .iter()
        .filter_map(|g| match &g.kind {
            GlobalKind::Local(InitExpr::RefFunc(_)) => Some(g.id()),
            _ => None,
        })
        .collect();

    for id in ids {
        if let GlobalKind::Local(InitExpr::RefFunc(i)) = &mut module.globals.get_mut(id).kind {
            if let Some(&id) = func_map.get(&i) {
                *i = id;
            }
        }
    }

    for &id in func_map.keys() {
        module.funcs.delete(id);
    }
}

mod parser;

use std::borrow::Cow;

use anyhow::{bail, Error};
use walrus::{CustomSection, Module, TypedCustomSectionId, ValType};

use crate::util::*;

const GODOT_WASM_BINDGEN_NAME: &str = "__godot_wasm_bindgen_data";

#[derive(Debug, Default, Clone)]
pub struct GodotWasmBindgenData {
    pub symbols: Vec<Symbol>,
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub version: [u8; 4],
    pub inner: SymbolType,
}

#[derive(Debug, Clone)]
pub enum SymbolType {
    ExportFunction(ExportFunction),
    ImportFunction(ImportFunction),
}

#[derive(Debug, Clone)]
pub struct ExportFunction {
    pub name: String,
    pub args: FunctionArgs,
}

#[derive(Debug, Clone)]
pub struct ImportFunction {
    pub module: String,
    pub name: String,
    pub args: FunctionArgs,
}

#[derive(Debug, Clone)]
pub struct FunctionArgs {
    pub params: Vec<ArgType>,
    pub results: Vec<ArgType>,
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum ArgType {
    U8 = 1,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    F32,
    F64,
    GodotValue,
}

impl From<ArgType> for ValType {
    fn from(v: ArgType) -> Self {
        match v {
            ArgType::U8
            | ArgType::I8
            | ArgType::U16
            | ArgType::I16
            | ArgType::U32
            | ArgType::I32 => Self::I32,
            ArgType::U64 | ArgType::I64 => Self::I64,
            ArgType::F32 => Self::F32,
            ArgType::F64 => Self::F64,
            ArgType::GodotValue => Self::Externref,
        }
    }
}

impl TryFrom<&[u8]> for GodotWasmBindgenData {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Error> {
        match parser::parse_bindgen_data(bytes) {
            Ok((_, ret)) => Ok(ret),
            Err(e) => bail!("{}", e),
        }
    }
}

impl CustomSection for GodotWasmBindgenData {
    fn name(&self) -> &str {
        GODOT_WASM_BINDGEN_NAME
    }

    fn data(&self, _: &walrus::IdsToIndices) -> Cow<[u8]> {
        let mut ret = Vec::new();

        for s in &self.symbols {
            let Symbol { version, inner } = s;

            let mut temp = Vec::new();

            match inner {
                SymbolType::ExportFunction(ExportFunction {
                    name,
                    args: FunctionArgs { params, results },
                }) => {
                    leb128::write::unsigned(&mut temp, 64).unwrap();
                    leb128::write::unsigned(&mut temp, name.len() as _).unwrap();
                    temp.extend_from_slice(name.as_bytes());

                    leb128::write::unsigned(&mut temp, params.len() as _).unwrap();
                    temp.extend(params.iter().map(|&v| v as u8));
                    leb128::write::unsigned(&mut temp, results.len() as _).unwrap();
                    temp.extend(results.iter().map(|&v| v as u8));
                }
                SymbolType::ImportFunction(ImportFunction {
                    module,
                    name,
                    args: FunctionArgs { params, results },
                }) => {
                    leb128::write::unsigned(&mut temp, 0).unwrap();
                    leb128::write::unsigned(&mut temp, module.len() as _).unwrap();
                    temp.extend_from_slice(module.as_bytes());
                    leb128::write::unsigned(&mut temp, name.len() as _).unwrap();
                    temp.extend_from_slice(name.as_bytes());

                    leb128::write::unsigned(&mut temp, params.len() as _).unwrap();
                    temp.extend(params.iter().map(|&v| v as u8));
                    leb128::write::unsigned(&mut temp, results.len() as _).unwrap();
                    temp.extend(results.iter().map(|&v| v as u8));
                }
            }

            ret.extend_from_slice(version);
            leb128::write::unsigned(&mut ret, temp.len() as _).unwrap();
            ret.extend_from_slice(&temp);
        }

        ret.into()
    }
}

#[derive(Debug)]
pub struct TargetFeatures {
    pub features: Vec<Feature>,
}

#[derive(Debug)]
pub struct Feature {
    pub enabled: bool,
    pub name: String,
}

impl TryFrom<&[u8]> for TargetFeatures {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Error> {
        match parser::parse_target_features(bytes) {
            Ok((_, ret)) => Ok(ret),
            Err(e) => bail!("{}", e),
        }
    }
}

impl CustomSection for TargetFeatures {
    fn name(&self) -> &str {
        "target_features"
    }

    fn data(&self, _: &walrus::IdsToIndices) -> Cow<[u8]> {
        let mut ret = Vec::new();
        leb128::write::unsigned(&mut ret, self.features.len() as _).unwrap();

        for f in &self.features {
            let Feature { enabled, name } = f;

            let mut temp: Vec<u8> = Vec::new();

            temp.extend_from_slice(name.as_bytes());
            tag_length(&mut temp);

            ret.push(if *enabled { 43 } else { 45 });
            ret.extend(temp);
        }

        ret.into()
    }
}

pub fn read_custom_data(
    module: &mut Module,
) -> Result<Option<TypedCustomSectionId<GodotWasmBindgenData>>, Error> {
    if let Some(data) = module.customs.remove_raw("target_features") {
        module
            .customs
            .add(TargetFeatures::try_from(&data.data as &[_])?);
    };

    let data = match module.customs.remove_raw(GODOT_WASM_BINDGEN_NAME) {
        Some(v) => v.data,
        None => return Ok(None),
    };
    let custom_section = GodotWasmBindgenData::try_from(&data as &[_])?;

    Ok(Some(module.customs.add(custom_section)))
}

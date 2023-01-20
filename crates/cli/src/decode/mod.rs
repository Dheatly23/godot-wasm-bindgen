mod parser;

use std::borrow::Cow;

use anyhow::{bail, Error};
use walrus::{CustomSection, Module, TypedCustomSectionId};

use crate::util::*;

const GODOT_WASM_BINDGEN_NAME: &str = "__godot_wasm_bindgen_data";

#[derive(Debug, Default, Clone)]
pub struct GodotWasmBindgenData {
    pub symbols: Vec<Symbol>,
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub version: [u8; 4],
    pub name: String,
    pub extra_data: Vec<u8>,
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
            let Symbol {
                version,
                name,
                extra_data,
            } = s;

            let mut temp: Vec<u8> = Vec::new();

            temp.extend_from_slice(name.as_bytes());
            tag_length(&mut temp);
            temp.extend_from_slice(extra_data);
            tag_length(&mut temp);

            ret.extend_from_slice(version);
            ret.extend(temp.into_iter());
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

use std::str::from_utf8;

use anyhow::bail;
use nom::bytes::complete::{tag, take, take_while_m_n};
use nom::combinator::{all_consuming, fail, map, map_opt, map_res};
use nom::multi::{length_count, length_data, length_value, many0};
use nom::sequence::{pair, tuple};
use nom::{IResult, Parser};

use super::{
    ArgType, ExportFunction, Feature, FunctionArgs, GodotWasmBindgenData, ImportFunction, Symbol,
    SymbolType, TargetFeatures,
};

fn not_end_byte(b: u8) -> bool {
    (b & 128) != 0
}

fn leb128_unsigned(input: &[u8]) -> IResult<&[u8], u64> {
    map_res(
        pair(
            take_while_m_n(0, 9, not_end_byte),
            take::<usize, &[u8], _>(1),
        ),
        |(s, b)| {
            debug_assert_eq!(b.len(), 1);
            debug_assert!(!not_end_byte(b[0]), "Invalid end byte value {:02x}", b[0]);

            let mut ret: u64 = (b[0] & 127).into();

            for &x in s.iter().rev() {
                debug_assert!(not_end_byte(x), "Invalid non-end byte value {x:02x}");
                ret = match ret.checked_mul(128) {
                    Some(v) => v | u64::from(x & 127),
                    None => bail!("Value overflow!"),
                };
            }

            Ok(ret)
        },
    )(input)
}

pub fn parse_bindgen_data(input: &[u8]) -> IResult<&[u8], GodotWasmBindgenData> {
    map(all_consuming(many0(parse_symbol)), |symbols| {
        GodotWasmBindgenData { symbols }
    })(input)
}

pub fn parse_symbol(input: &[u8]) -> IResult<&[u8], Symbol> {
    fn parse_version(input: &[u8]) -> IResult<&[u8], [u8; 4]> {
        let (input, v) = take(4usize)(input)?;
        debug_assert_eq!(v.len(), 4);
        Ok((input, v.try_into().unwrap()))
    }

    map(
        tag(&[1, 0, 0, 0]).and_then(parse_version).and(length_value(
            leb128_unsigned,
            all_consuming(parse_symbol_type),
        )),
        |(version, inner)| Symbol { version, inner },
    )(input)
}

pub fn parse_symbol_type(input: &[u8]) -> IResult<&[u8], SymbolType> {
    fn switch_symbol(v: u64) -> impl Fn(&[u8]) -> IResult<&[u8], SymbolType> {
        move |i| match v {
            64 => map(parse_export_function, |v| SymbolType::ExportFunction(v))(i),
            0 => map(parse_import_function, |v| SymbolType::ImportFunction(v))(i),
            _ => fail(i),
        }
    }

    leb128_unsigned.flat_map(switch_symbol).parse(input)
}

pub fn parse_export_function(input: &[u8]) -> IResult<&[u8], ExportFunction> {
    map(
        tuple((
            map_res(length_data(leb128_unsigned), from_utf8),
            parse_function_args,
        )),
        |(name, args)| ExportFunction {
            name: name.into(),
            args,
        },
    )(input)
}

pub fn parse_import_function(input: &[u8]) -> IResult<&[u8], ImportFunction> {
    map(
        tuple((
            map_res(length_data(leb128_unsigned), from_utf8),
            map_res(length_data(leb128_unsigned), from_utf8),
            parse_function_args,
        )),
        |(module, name, args)| ImportFunction {
            module: module.into(),
            name: name.into(),
            args,
        },
    )(input)
}

pub fn parse_function_args(input: &[u8]) -> IResult<&[u8], FunctionArgs> {
    fn arg_type(v: &[u8]) -> Option<ArgType> {
        debug_assert_eq!(v.len(), 1);
        match v[0] {
            1 => Some(ArgType::U8),
            2 => Some(ArgType::I8),
            3 => Some(ArgType::U16),
            4 => Some(ArgType::I16),
            5 => Some(ArgType::U32),
            6 => Some(ArgType::I32),
            7 => Some(ArgType::U64),
            8 => Some(ArgType::I64),
            9 => Some(ArgType::F32),
            10 => Some(ArgType::F64),
            11 => Some(ArgType::GodotValue),
            _ => None,
        }
    }

    map(
        tuple((
            length_value(leb128_unsigned, many0(map_opt(take(1usize), arg_type))),
            length_value(leb128_unsigned, many0(map_opt(take(1usize), arg_type))),
        )),
        |(params, results)| FunctionArgs { params, results },
    )(input)
}

pub fn parse_target_features(input: &[u8]) -> IResult<&[u8], TargetFeatures> {
    map(
        all_consuming(length_count(leb128_unsigned, parse_feature)),
        |features| TargetFeatures { features },
    )(input)
}

pub fn parse_feature(input: &[u8]) -> IResult<&[u8], Feature> {
    map(
        tuple((
            map_res(take(1usize), |e: &[u8]| {
                debug_assert_eq!(e.len(), 1);

                match e[0] {
                    43 => Ok(true),
                    45 => Ok(false),
                    v => bail!("Unknown flag {}", char::from_u32(v.into()).unwrap_or('?')),
                }
            }),
            map_res(length_data(leb128_unsigned), from_utf8),
        )),
        |(enabled, name)| Feature {
            enabled,
            name: name.into(),
        },
    )(input)
}

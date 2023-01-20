use std::str::from_utf8;

use anyhow::{bail, Error};
use nom::bytes::complete::{take, take_while_m_n};
use nom::combinator::{all_consuming, map, map_res, rest};
use nom::multi::{length_count, length_data, length_value, many0};
use nom::sequence::{pair, tuple};
use nom::IResult;

use super::{Feature, GodotWasmBindgenData, Symbol, TargetFeatures};

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
    map_res(
        tuple((
            take(4usize),
            length_value(leb128_unsigned, tuple((length_data(leb128_unsigned), rest))),
        )),
        |(version, (name, rest))| -> Result<_, Error> {
            debug_assert_eq!(version.len(), 4);

            Ok(Symbol {
                version: version.try_into()?,
                name: from_utf8(name)?.into(),
                extra_data: rest.into(),
            })
        },
    )(input)
}

pub fn parse_target_features(input: &[u8]) -> IResult<&[u8], TargetFeatures> {
    map(
        all_consuming(length_count(leb128_unsigned, parse_feature)),
        |features| TargetFeatures { features },
    )(input)
}

pub fn parse_feature(input: &[u8]) -> IResult<&[u8], Feature> {
    map_res(
        tuple((take(1usize), length_data(leb128_unsigned))),
        |(e, name)| {
            debug_assert_eq!(e.len(), 1);

            Ok(Feature {
                enabled: match e[0] {
                    43 => true,
                    45 => false,
                    v => bail!("Unknown flag {}", char::from_u32(v.into()).unwrap_or('?')),
                },
                name: from_utf8(name)?.into(),
            })
        },
    )(input)
}

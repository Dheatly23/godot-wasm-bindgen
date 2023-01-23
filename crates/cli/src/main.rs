mod decode;
mod runtime;
mod substitution;
mod util;

use std::path::PathBuf;

use anyhow::Error;
use clap::Parser;
use walrus::passes::gc;
use walrus::{FunctionKind, Module};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about)]
struct Args {
    #[arg(short, long, help = "Output file name")]
    output: Option<PathBuf>,

    #[arg(help = "Input file name")]
    file: PathBuf,
}

fn main() -> Result<(), Error> {
    let args = Args::parse();

    let mut module = Module::from_file(args.file)?;

    let custom_id = decode::read_custom_data(&mut module)?;
    println!("{:?}", custom_id.and_then(|v| module.customs.get(v)));

    let runtime = runtime::add_runtime(&mut module)?;

    if let Some(custom_data) = custom_id.and_then(|id| module.customs.delete(id)) {
        substitution::substitute_exports(&mut module, &custom_data, &runtime)?;
        substitution::substitute_imports(&mut module, &custom_data, &runtime)?;
    }

    gc::run(&mut module);

    for f in module.funcs.iter() {
        if let FunctionKind::Import(v) = &f.kind {
            println!(
                "{:?} {}: {:?} import {:?}",
                f.id(),
                match f.name.as_ref() {
                    Some(v) => v,
                    None => "",
                },
                module.types.get(v.ty),
                module.imports.get(v.import),
            );
        } else {
            println!(
                "{:?} {}: {:?} local",
                f.id(),
                match f.name.as_ref() {
                    Some(v) => v,
                    None => "",
                },
                module.types.get(f.ty()),
            );
        }
    }

    if let Some(features) = module.customs.get_typed_mut::<decode::TargetFeatures>() {
        features.features.push(decode::Feature {
            enabled: true,
            name: String::from("multi-memory"),
        });
    }

    if let Some(output) = &args.output {
        module.emit_wasm_file(output)?;
    }

    Ok(())
}

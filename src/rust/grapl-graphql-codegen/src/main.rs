use std::{
    io::Read,
    path::PathBuf,
};

use color_eyre::eyre::{
    Result,
    WrapErr,
};
use graphql_parser::schema::parse_schema;
use structopt::StructOpt;

pub mod as_static_python;
pub mod conflict_resolution;
pub mod constants;
pub mod edge;
pub mod edge_rel;
pub mod errors;
pub mod external_helpers;
pub mod field_type;
pub mod identification_algorithm;
pub mod identity_predicate_type;
pub mod node_predicate;
pub mod node_type;
pub mod predicate_type;

#[derive(Debug, StructOpt)]
#[structopt(name = "grapl-graphql-codegen", about = "Codegen for Grapl plugins")]
struct Opt {
    /// Input file, stdin if not present
    #[structopt(short = "i", long = "input", parse(from_os_str), env)]
    input: Option<PathBuf>,

    /// Output file, stdout if not present
    #[structopt(short = "o", long = "output", parse(from_os_str), env)]
    output: Option<PathBuf>,

    /// Do not emit any generated code - useful with 'validate'
    #[structopt(long = "no-emit", parse(from_flag))]
    no_emit: bool,

    /// Build the code with line numbers
    #[structopt(long = "line-num", parse(from_flag))]
    line_num: bool,

    /// Generated code will be passed to the system Python interpreter, and mypy will be executed
    /// against the code as well
    #[structopt(long = "validate", parse(from_flag))]
    validate: bool,
}

fn read_in_schema(input: &Option<PathBuf>) -> Result<String> {
    match input {
        Some(path) => Ok(std::fs::read_to_string(path)
            .context(format!("Failed to read from file: {:?}", path))?),
        None => {
            let mut buf = String::with_capacity(256);
            std::io::stdin()
                .lock()
                .read_to_string(&mut buf)
                .context("Failed to read from stdin")?;
            Ok(buf)
        }
    }
}

fn standin_imports() -> String {
    let mut code = String::new();
    code.push_str("from __future__ import annotations\n");
    code.push_str("from dataclasses import dataclass");
    code.push_str("from typing import Optional, Any, Set, List, Dict, Tuple\n");
    code.push_str("import grapl_analyzerlib\n");
    code.push_str("import grapl_analyzerlib.node_types\n");
    code.push_str("import grapl_analyzerlib.nodes.entity\n");
    code.push_str("import grapl_analyzerlib.queryable\n");
    code
}

#[tracing::instrument]
fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    color_eyre::install()?;

    let opt = Opt::from_args();

    tracing::debug!(message="Executing grapl-graphql-codegen", options=?opt);
    let raw_schema = read_in_schema(&opt.input)?;
    let document = parse_schema(&raw_schema)?;
    let node_types = node_type::parse_into_node_types(&document).expect("Failed");

    let mut all_code = String::with_capacity(1024 * node_types.len());
    all_code.push_str(&standin_imports());
    for node_type in node_types {
        let pycode = node_type.generate_python_code();
        all_code.push_str(&pycode);
    }

    if opt.validate {
        grapl_graphql_codegen::external_helpers::validate_code(&all_code)?;
    }

    // If `no_emit` is set, return early
    if opt.no_emit {
        return Ok(());
    }

    // `output` being none implies we should write to stdout
    if opt.output.is_none() {
        for (i, s) in all_code.split("\n").enumerate() {
            if opt.line_num {
                // https://doc.rust-lang.org/std/fmt/#fillalignment
                // right-pad the line numbers
                print!("{: <4}: ", i + 1);
            }
            println!("{}", s);
        }
    }

    if let Some(ref path) = opt.output {
        if !opt.no_emit {
            std::fs::write(path, all_code.as_bytes())?;
        } else {
            tracing::debug!(
                message="output specified, but no_emit is true - skipping",
                output_path=?path,
                no_emit=?opt.no_emit,
            );
        }
    }

    Ok(())
}

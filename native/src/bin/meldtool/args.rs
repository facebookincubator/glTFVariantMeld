// Copyright (c) Facebook, Inc. and its affiliates. All Rights Reserved
//

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use clap::{crate_authors, crate_version, App, Arg};

#[derive(Debug, PartialEq)]
pub enum Verbosity {
    Quiet,
    Normal,
    Verbose,
}

#[derive(Debug)]
pub struct WorkOrder {
    pub source_assets: SourceAssets,
    pub output_path: PathBuf,
    pub verbosity: Verbosity,
}

impl WorkOrder {
    pub fn verbose(&self) -> bool {
        self.verbosity == Verbosity::Verbose
    }
    pub fn quiet(&self) -> bool {
        self.verbosity == Verbosity::Quiet
    }
}

#[derive(Debug)]
pub struct SourceAssets {
    pub base: SourceAsset,
    pub melds: Vec<SourceAsset>,
}

#[derive(Debug)]
pub struct SourceAsset {
    pub path: PathBuf,
    pub tag: Option<String>,
}

pub fn parse_args() -> WorkOrder {
    let matches = App::new("glTFVariantMeld")
        .author(crate_authors!())
        .version(crate_version!())
        .arg(
            Arg::with_name("base")
                .short("b")
                .long("base")
                .required(true)
                .takes_value(true)
                .value_name("FILE")
                .help("the base source asset into which to meld"),
        )
        .arg(
            Arg::with_name("tag")
                .short("t")
                .long("tagged-as")
                .takes_value(true)
                .multiple(true)
                .value_name("TAG")
                .help("a variant tag representing the preceding source asset"),
        )
        .arg(
            Arg::with_name("meld")
                .short("m")
                .long("meld")
                .takes_value(true)
                .multiple(true)
                .value_name("FILE")
                .help("a source asset to meld into the base"),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .required(true)
                .takes_value(true)
                .value_name("FILE")
                .help("the name of the output file"),
        )
        .arg(
            Arg::with_name("force")
                .short("f")
                .long("force")
                .takes_value(false)
                .help("overwrite output file if it exists"),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .takes_value(false)
                .help("output more detailed progress"),
        )
        .arg(
            Arg::with_name("quiet")
                .short("q")
                .long("quiet")
                .takes_value(false)
                .help("suppress all informational output"),
        )
        .get_matches();

    let source_assets = parse_source_assets(&matches);

    let force = matches.occurrences_of("force") > 0;
    let output_path = &matches.value_of("output").unwrap();
    if let Ok(metadata) = fs::metadata(output_path) {
        if metadata.is_dir() {
            eprintln!("Error: Output path is a directory: {}", output_path);
            std::process::exit(1);
        } else if metadata.is_file() && !force {
            eprintln!(
                "Error: Output path exists (use -f to overwrite): {}",
                output_path
            );
            std::process::exit(1);
        }
    }
    let output_path = PathBuf::from(output_path);

    let verbosity = if matches.occurrences_of("verbose") > 0 {
        Verbosity::Verbose
    } else if matches.occurrences_of("quiet") > 0 {
        Verbosity::Quiet
    } else {
        Verbosity::Normal
    };

    WorkOrder {
        source_assets,
        output_path,
        verbosity,
    }
}

fn parse_source_assets(matches: &clap::ArgMatches) -> SourceAssets {
    let base = matches.value_of("base").unwrap();
    let base_ix = matches.index_of("base").unwrap();

    let tag_map = if let Some(tags) = matches.values_of("tag") {
        let ix = matches.indices_of("tag").unwrap();
        ix.zip(tags).collect()
    } else {
        HashMap::new()
    };

    let mk_asset = |pathstr, ix| {
        let path = PathBuf::from(pathstr);
        if path.exists() {
            let tag = tag_map.get(&(ix + 2)).map(|t| (*t).to_owned());
            SourceAsset { path, tag }
        } else {
            eprintln!("Error: Couldn't open file: {}", pathstr);
            std::process::exit(1);
        }
    };

    let base = mk_asset(base, base_ix);

    let melds = if let Some(melds) = matches.values_of("meld") {
        let ix = matches.indices_of("meld").unwrap();
        melds
            .zip(ix)
            .map(|(meld, meld_ix)| mk_asset(meld, meld_ix))
            .collect()
    } else {
        vec![]
    };

    SourceAssets { base, melds }
}

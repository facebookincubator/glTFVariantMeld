// Copyright (c) Facebook, Inc. and its affiliates. All Rights Reserved
//

#[macro_use]
extern crate clap;

use std::collections::HashMap;
use std::fs::File;

use clap::{App, Arg};

fn main() {
    let matches = App::new("glTFVariantMeld")
        .author(crate_authors!())
        .version(crate_version!())
        .arg(
            Arg::with_name("base")
                .short("b")
                .long("base")
                .takes_value(true)
                .required(true)
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
                .takes_value(true)
                .value_name("FILE")
                .help("the name of the output file"),
        )
        .get_matches();

    let source_assets = parse_source_assets(matches);
    println!("Source assets: {:#?}", source_assets);
}

fn parse_source_assets(matches: clap::ArgMatches) -> SourceAssets {
    let base = matches.value_of("base").unwrap();
    let base_ix = matches.index_of("base").unwrap();

    let tag_map = if let Some(tags) = matches.values_of("tag") {
        let ix = matches.indices_of("tag").unwrap();
        ix.zip(tags).collect()
    } else {
        HashMap::new()
    };

    let mk_asset = |path, ix| {
        if let Ok(file) = File::open(path) {
            let tag = tag_map.get(&(ix + 2)).map(|t| (*t).to_owned());
            SourceAsset { file, tag }
        } else {
            eprintln!("Error: Couldn't open file: {}", path);
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

#[derive(Debug)]
struct SourceAssets {
    base: SourceAsset,
    melds: Vec<SourceAsset>,
}

#[derive(Debug)]
struct SourceAsset {
    file: File,
    tag: Option<String>,
}

//   let result = parse_input(0);
//   console.log("Initial asset:");
//   describe_asset(result);

//   for (let ix = 1; ix < inputs.length; ix++) {
//     console.log();
//     result = wasmpkg.VariationalAsset.wasm_meld(result, parse_input(ix));
//     console.log("New melded result:");
//     describe_asset(result);
//   }

//   let output_glb = result.wasm_glb();
//   writeFileSync(output, output_glb);

//   console.log("Success! %d bytes written to '%s'.", output_glb.length, output);
// }

// function describe_asset(asset: VariationalAsset) {
//   console.log("             Total file size: " + size(asset.wasm_glb().length));
//   let total = asset.wasm_metadata().total_sizes().texture_bytes;
//   let variational = asset.wasm_metadata().variational_sizes().texture_bytes;
//   console.log("          Total texture data: " + size(total));
//   console.log("  Of which is depends on tag: " + size(variational));
// }

// function size(byte_count: number): string {
//   if (byte_count < 1000000) {
//     return (byte_count / 1000).toFixed(1) + " kB";
//   }
//   return (byte_count / 1000000).toFixed(1) + " MB";
// }

// // Synchronously read the contents of a file, and ascertain that it's a GLB file.
// function readAndValidate(file: string): Buffer {
//   let contents = readFileSync(file);
//   let first_word = contents.readUIntLE(0, 4);
//   if (first_word === GLB_MAGIC) {
//     return contents;
//   }
//   console.error("File %s is not a GLB file: starts with 0x%s.", file, first_word.toString(16));
//   process.exit(1);
// }

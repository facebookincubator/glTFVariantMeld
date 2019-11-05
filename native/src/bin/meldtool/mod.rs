// Copyright (c) Facebook, Inc. and its affiliates. All Rights Reserved
//

extern crate clap;

extern crate gltf_variant_meld;

use std::fs;

use gltf_variant_meld::{Result, VariationalAsset};

mod args;
use args::parse_args;
pub use args::{SourceAsset, SourceAssets, WorkOrder};

fn main() {
    let work_order = parse_args();

    if let Err(err) = process(work_order) {
        eprintln!("Error: {}", err);
    }
}

fn process(work_order: WorkOrder) -> Result<()> {
    let base = read_asset(&work_order.source_assets.base)?;
    if work_order.verbose() {
        println!("Base asset:");
        describe_asset(&base);
    }

    let mut result = base;
    for meld in &work_order.source_assets.melds {
        let meld = read_asset(meld)?;
        result = VariationalAsset::meld(&result, &meld)?;
        if work_order.verbose() {
            println!("New melded result:");
            describe_asset(&result);
        }
    }

    fs::write(&work_order.output_path, result.glb())
        .map_err(|e| format!("Couldn't write output file: {}", e))?;

    if !work_order.quiet() {
        println!(
            "Success! {} bytes written to '{}'.",
            result.glb().len(),
            work_order.output_path.to_str().unwrap_or("<error>"),
        );
    }
    Ok(())
}

fn read_asset(asset: &SourceAsset) -> Result<VariationalAsset> {
    Ok(VariationalAsset::from_file(
        &asset.path,
        asset.tag.as_ref(),
    )?)
}

fn describe_asset(asset: &VariationalAsset) {
    println!("             Total file size: {}", size(asset.glb().len()));
    let total = asset.metadata().total_sizes().texture_bytes;
    let variational = asset.metadata().variational_sizes().texture_bytes;
    println!("          Total texture data: {}", size(total));
    println!("  Of which is depends on tag: {}", size(variational));
}

fn size(byte_count: usize) -> String {
    if byte_count < 1000000 {
        format!("{:.01} kB", byte_count / 1000)
    } else {
        format!("{:.01} MB", byte_count / 1000000)
    }
}

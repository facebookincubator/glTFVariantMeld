// Copyright (c) Facebook, Inc. and its affiliates. All Rights Reserved
//

extern crate clap;

extern crate gltf_variant_meld;

use std::fs;

use gltf_variant_meld::VariationalAsset;

mod args;
use args::parse_args;
pub use args::{SourceAsset, SourceAssets, WorkOrder};

mod error_handling;
use error_handling::Result;

fn main() {
    let work_order = parse_args();

    if let Err(err) = process(work_order) {
        eprintln!("Error: {}", err);
    }
}

fn process(work_order: WorkOrder) -> Result<()> {
    let base = &work_order.source_assets.base;
    let base_asset = read_asset(base)?;
    if work_order.verbose() {
        println!(
            "Parsed base asset [{}]: {:?}",
            size(base_asset.glb().len()),
            base.path.file_name()?
        );
    }

    let mut result = base_asset;
    for meld in &work_order.source_assets.melds {
        let meld_asset = read_asset(meld)?;
        if work_order.verbose() {
            println!();
            println!(
                "Melding in asset [{}]: {:?}",
                size(meld_asset.glb().len()),
                meld.path.file_name()?
            );
        }
        let old_size = result.glb().len();
        result = VariationalAsset::meld(&result, &meld_asset)?;
        if work_order.verbose() {
            let new_size = result.glb().len();
            println!(
                "Total post-meld size increase: {}",
                size(new_size - old_size)
            );
        }
    }

    if work_order.verbose() {
        println!();
        println!("Final asset size: {}", size(result.glb().len()));
        describe_textures(&result)?;
        println!();
    }

    fs::write(&work_order.output_path, result.glb())
        .map_err(|e| format!("Couldn't write output file: {}", e))?;

    if !work_order.quiet() {
        println!(
            "{} bytes written to '{}'.",
            result.glb().len(),
            work_order.output_path.to_str()?,
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

fn describe_textures(asset: &VariationalAsset) -> Result<()> {
    let metadata = asset.metadata();
    let shared_textures =
        metadata.total_sizes().texture_bytes() - metadata.variational_sizes().texture_bytes();

    println!("Active variational texture, by tag:");
    let mut tags: Vec<_> = metadata.tags().iter().collect();
    tags.sort();
    for &tag in &tags {
        let textures_for_tag = metadata.tag_sizes(tag)?.texture_bytes() - shared_textures;
        println!("{:>30}: {}", format!("[{}]", tag), size(textures_for_tag));
    }
    println!("Textures shared by all variants: {}", size(shared_textures));
    Ok(())
}

fn size(byte_count: usize) -> String {
    if byte_count < 1000000 {
        format!("{:.01} kB", (byte_count as f64) / 1000.0)
    } else {
        format!("{:.01} MB", (byte_count as f64) / 1000000.0)
    }
}

// Copyright (c) Facebook, Inc. and its affiliates. All Rights Reserved
//

extern crate assets;
extern crate gltf_variant_meld;

use std::collections::{HashMap};

use spectral::prelude::*;

use gltf_variant_meld::{Tag, VariationalAsset, WorkAsset};

use assets::*;

#[test]
fn test_parse_simple_variational() {
    let (tag_1, tag_2) = (
        Tag::from("tag_1"),
        Tag::from("tag_2"),
    );

    let mut variant_ix_lookup = HashMap::new();
    variant_ix_lookup.insert(0, tag_1.to_owned());
    variant_ix_lookup.insert(1, tag_2.to_owned());

    let asset_result =
        VariationalAsset::from_file(ASSET_PINECONE_VARIATIONAL(), Some(&tag_1));
    assert_that!(asset_result).is_ok();
    let asset = asset_result.unwrap();
    let asset = WorkAsset::from_slice(asset.glb(), Some(&tag_2), None)
        .or_else(|e| Err(e.to_string()))
        .expect("glTF re-parse failure");

    let mesh = asset.meshes().get(0).expect("No meshes!?");
    let primitive = mesh
        .primitives
        .get(0)
        .expect("No primitives in first mesh!");
    let extracted_map = gltf_variant_meld::extension::extract_variant_map(&primitive, &variant_ix_lookup)
        .expect("Failed to extract variant map from mesh primitive.");

    assert_that!(extracted_map).has_length(2);
    assert_that!(extracted_map.keys()).contains_all_of(&vec![&tag_1, &tag_2]);
}

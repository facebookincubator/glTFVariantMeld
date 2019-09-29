// Copyright (c) Facebook, Inc. and its affiliates. All Rights Reserved
//

extern crate assets;
extern crate gltf_variant_meld;

use spectral::prelude::*;

use gltf_variant_meld::{Tag, VariationalAsset, WorkAsset};

use assets::*;

#[test]
fn test_parse_simple_variational() {
    let (tag_default, tag_1, tag_2) = (
        Tag::from("tag_default"),
        Tag::from("tag_1"),
        Tag::from("tag_2"),
    );
    let asset_result =
        VariationalAsset::from_file(ASSET_PINECONE_VARIATIONAL(), Some(&tag_default));
    assert_that!(asset_result).is_ok();
    let asset = asset_result.unwrap();
    let asset = WorkAsset::from_slice(asset.glb(), None, None)
        .or_else(|e| Err(e.to_string()))
        .expect("glTF re-parse failure");

    let mesh = asset.meshes().get(0).expect("No meshes!?");
    let primitive = mesh
        .primitives
        .get(0)
        .expect("No primitives in first mesh!");
    let extracted_map = gltf_variant_meld::extension::extract_variant_map(&primitive)
        .expect("Failed to extract variant map from mesh primitive.");

    assert_that!(extracted_map).has_length(3);
    assert_that!(extracted_map.keys()).contains_all_of(&vec![&tag_default, &tag_1, &tag_2]);
}

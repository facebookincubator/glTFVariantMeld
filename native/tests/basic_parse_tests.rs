// Copyright (c) Facebook, Inc. and its affiliates. All Rights Reserved
//

extern crate assets;
extern crate gltf_variant_meld;

use std::iter::FromIterator;

use spectral::prelude::*;

use assets::*;

use gltf::Gltf;

use gltf_variant_meld::{Tag, VariationalAsset};

#[test]
fn test_tiny_parse() {
    let json = r#"
    {
        "asset": {
            "version": "2.0"
        }
    }
    "#;
    let asset = VariationalAsset::from_slice(json.as_bytes(), Some(&Tag::from("tag")), None)
        .expect("glTF parse failure");

    let asset = Gltf::from_slice(asset.glb())
        .or_else(|e| Err(e.to_string()))
        .expect("glTF re-parse failure");

    assert_that!(Vec::from_iter(asset.accessors())).has_length(0);
    assert_that!(Vec::from_iter(asset.extensions_used())).has_length(1);

    println!("JSON as parsed: {:#?}", asset.document.clone().into_json());
}

#[test]
fn test_larger_parse() {
    let asset = VariationalAsset::from_file(ASSET_PINECONE_SHINY(), Some(&Tag::from("tag")))
        .expect("glTF import failure");
    let asset = &Gltf::from_slice(asset.glb())
        .or_else(|e| Err(e.to_string()))
        .expect("glTF re-parse failure");

    assert_that!(Vec::from_iter(asset.accessors())).has_length(5);
}

#[test]
fn test_pinecone_comparison() {
    let tests = vec![
        (ASSET_PINECONE_MATTE(), Tag::from("matte"), 0.2, 0.8),
        (ASSET_PINECONE_SHINY(), Tag::from("shiny"), 0.8, 0.2),
    ];
    for test in tests {
        let pinecone =
            VariationalAsset::from_file(test.0, Some(&test.1)).expect("glTF import failure");

        let pinecone = &Gltf::from_slice(pinecone.glb())
            .or_else(|e| Err(e.to_string()))
            .expect("glTF re-parse failure");

        assert_that!(Vec::from_iter(pinecone.accessors())).has_length(5);

        let pbr = pinecone
            .materials()
            .nth(0)
            .unwrap()
            .pbr_metallic_roughness();
        assert_that!(pbr.metallic_factor()).is_equal_to(test.2);
        assert_that!(pbr.roughness_factor()).is_equal_to(test.3);
    }
}

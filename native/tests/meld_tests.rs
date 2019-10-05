// Copyright (c) Facebook, Inc. and its affiliates. All Rights Reserved
//

extern crate assets;
extern crate gltf_variant_meld;

use spectral::prelude::*;

use assets::*;

use gltf_variant_meld::{Tag, VariationalAsset};

#[test]
fn test_pinecone_meld() {
    let (shiny, matte, tinted) = (Tag::from("shiny"), Tag::from("matte"), Tag::from("tinted"));

    let load_asset = |path, tag| {
        VariationalAsset::from_file(path, Some(tag)).expect("VariationalAsset::from_file() failure")
    };
    let meld_assets = |base, other| {
        VariationalAsset::meld(base, other).expect("VariationalAsset::meld() failure")
    };

    // helper lambdas
    let test = |asset: &VariationalAsset, default_tag, tags: Vec<&Tag>| {
        assert_that!(asset.default_tag()).is_equal_to(default_tag);
        assert_that!(asset.metadata().tags().iter()).contains_all_of(&tags);
        assert_that!(asset.metadata().tags().iter().count()).is_equal_to(tags.len());
    };

    let matte_pinecone = load_asset(ASSET_PINECONE_MATTE(), &matte);
    let shiny_pinecone = load_asset(ASSET_PINECONE_SHINY(), &shiny);
    let matte_shiny_pinecone = meld_assets(&matte_pinecone, &shiny_pinecone);
    test(&matte_shiny_pinecone, &matte, vec![&matte, &shiny]);

    let tinted_pinecone = load_asset(ASSET_PINECONE_TINTED(), &tinted);
    let matte_shiny_tinted = meld_assets(&matte_shiny_pinecone, &tinted_pinecone);
    test(&matte_shiny_tinted, &matte, vec![&matte, &shiny, &tinted]);

    let tinted_matte_shiny = meld_assets(&tinted_pinecone, &matte_shiny_pinecone);
    test(&tinted_matte_shiny, &tinted, vec![&matte, &shiny, &tinted]);
}

#[test]
fn test_teapot_meld() {
    let (camo_pink_bronze, camo_pink_silver, green_pink_bronze, green_pink_silver) = (
        Tag::from("camo_pink_bronze"),
        Tag::from("camo_pink_silver"),
        Tag::from("green_pink_bronze"),
        Tag::from("green_pink_silver"),
    );

    // helper lambdas
    let load_and_test = |path, tag, ts| {
        let asset = VariationalAsset::from_file(path, Some(tag)).expect("glTF import failure");

        assert_that!(asset.metadata().total_sizes().texture_bytes()).is_equal_to(ts);
        assert_that!(asset.metadata().variational_sizes().texture_bytes()).is_equal_to(0);
        return asset;
    };

    let meld_and_test = |base, meld, ts| {
        let melded = VariationalAsset::meld(base, meld).expect("VariationalAsset::meld() failure");
        let metadata = melded.metadata();
        assert_that!(metadata.total_sizes().texture_bytes()).is_equal_to(ts);
        assert_that!(metadata.variational_sizes().texture_bytes()).is_equal_to(ts);
        return melded;
    };

    let test_tag = |asset: &VariationalAsset, tag, ts| {
        assert_that!(asset.metadata().tag_sizes(tag).unwrap().texture_bytes()).is_equal_to(ts)
    };

    // actual test logic begins here
    let base_pot = load_and_test(ASSET_TEAPOT_CAMO_PINK_BRONZE(), &camo_pink_bronze, 227318);
    let meld_pot = load_and_test(ASSET_TEAPOT_CAMO_PINK_SILVER(), &camo_pink_silver, 227318);

    let melded = meld_and_test(&base_pot, &meld_pot, 227318);
    test_tag(&melded, &camo_pink_bronze, 227318);
    test_tag(&melded, &camo_pink_silver, 227318);

    // add in a pot with the green texture
    let base_pot = melded;
    let meld_pot = load_and_test(ASSET_TEAPOT_GREEN_PINK_SILVER(), &green_pink_silver, 337020);
    let melded = meld_and_test(&base_pot, &meld_pot, 564338);
    test_tag(&melded, &camo_pink_bronze, 227318);
    test_tag(&melded, &camo_pink_silver, 227318);
    test_tag(&melded, &green_pink_silver, 337020);

    // finally a fourth variant that should add no new texture
    let base_pot = melded;
    let meld_pot = load_and_test(ASSET_TEAPOT_GREEN_PINK_BRONZE(), &green_pink_bronze, 337020);
    let melded = meld_and_test(&base_pot, &meld_pot, 564338);
    test_tag(&melded, &camo_pink_bronze, 227318);
    test_tag(&melded, &camo_pink_silver, 227318);
    test_tag(&melded, &green_pink_silver, 337020);
    test_tag(&melded, &green_pink_bronze, 337020);
}

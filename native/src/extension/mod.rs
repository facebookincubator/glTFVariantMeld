// Copyright (c) Facebook, Inc. and its affiliates. All Rights Reserved
//

//! Implementation of our
//! [`FB_variant_mapping`](https://github.com/zellski/glTF/blob/ext/zell-fb-asset-variants/extensions/2.0/Vendor/FB_material_variants/README.md)
//! extension.
//!
//! We're specifically concerned with reading and writing values that are meaningful from
//! the point of view of i.e. `WorkAsset` into a glTF format, and especially the abstraction
//! we get from the `gltf` crates.

use gltf::json::Root;

const FB_MATERIAL_VARIANTS: &str = "FB_material_variants";

mod on_root;
pub use on_root::{get_validated_extension_tag, set_extension_tag};

mod on_primitive;
pub use on_primitive::{extract_variant_map, write_variant_map};

/// Updates the `extensions_used` glTF property with the name of our extension.
///
pub fn install(root: &mut Root) {
    let used = &mut root.extensions_used;
    if !used.contains(&String::from(FB_MATERIAL_VARIANTS)) {
        used.push(String::from(FB_MATERIAL_VARIANTS));
    }
}

// Copyright (c) Facebook, Inc. and its affiliates. All Rights Reserved
//

//! Implementation of our `FB_variant_mapping` extension.
//!
//! We're specifically concerned with reading and writing values that are meaningful from
//! the point of view of i.e. `WorkAsset` into a glTF format, and especially the abstraction
//! we get from the `gltf` crates.
//!
//! TODO: This currently works on `extras` rather than `extensions`, because support for the
//! latter is not as well-developed in the gltf crates. We may need to work against a fork of
//! those to get proper extension support.

mod on_root;
pub use on_root::{get_validated_extension_tag, set_extension_tag};

mod on_primitive;
pub use on_primitive::{extract_variant_map, write_variant_map};

// Copyright (c) Facebook, Inc. and its affiliates. All Rights Reserved
//

#![warn(missing_docs)]

//! # glTFVariantMeld
//!
//! ### Introduction
//!
//! This library exists to do one single thing: meld multiple glTF assets, each representing a
//! different *variant* of some basic model, into a single, compact format, implemented as a formal
//! glTF extension.
//!
//! For a practical canonical use case, take the common case of a retail product that's available in
//! a range of colours, and an application that lets a prospective customer switch between these
//! different variants without latency or stutters.
//!
//! We're making this internal tool publicly available with the hope of bringing the ecosystem
//! together around a common, open format, the lingua franca of variational 3D assets. One day,
//! perhaps digital content creation tools will include ways to export variational assets natively.
//! Until that day, this is how we're doing it.
//!
//! In this prerelease version, the tool produces files with the Khronos extension
//! [`KHR_materials_variants`](https://github.com/zellski/glTF/blob/ext/zell-fb-asset-variants/extensions/2.0/Khronos/KHR_materials_variants/README.md).
//! We are hopeful that the glTF community will find speedy consensus around a fully
//! ratified extension, e.g. `KHR_material_variants`.
//!
//! At present, we offer a simple command line interface. Our aspirational roadmap includes the
//! development of a web app which would leverage WebAssembly to run entirely in the browser.
//!
//! ### Technical Requirements
//!
//! For assets to be meldable, they must be logically identical, i.e. contain the same meshes – and
//! vary only in what materials are assigned to those meshes. The tool will complain if it finds
//! disrepancies between the source assets that are too confusing for it to work around.
//!
//! During the melding process, all common data is shared, whereas varying material definitions and
//! textures are copied as needed. Parts of the assets that don't vary are left untouched.
//!
//! Each source asset brought into the tool is identified by a *tag*, a short string, and it's
//! these same tags that are later used to trigger different runtime apperances.

extern crate gltf;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate sha1;
extern crate spectral;

/// Tags are short identifiers used to switch between different mesh primitive materials.
pub type Tag = String;

/// Our library-wide error type is (as yet) a simple string.
pub type Error = String;
/// Convenience type for a Result using our Error.
pub type Result<T> = ::std::result::Result<T, crate::Error>;

/// The JSON/Serde implementation of `KHR_materials_variants`.
pub mod extension;

/// The VarationalAsset struct and associated functionality.
pub mod variational_asset;
pub use variational_asset::{AssetSizes, Metadata, VariationalAsset};

/// The internal workhorse WorkAsset struct & functionality.
pub mod work_asset;
pub use work_asset::WorkAsset;

pub mod glb;
pub use glb::GlbChunk;

pub mod gltfext;
pub use gltfext::*;

/// Mapping glTF objects to unique keys for melding purposes.
pub mod meld_keys;
pub use meld_keys::{Fingerprint, MeldKey};

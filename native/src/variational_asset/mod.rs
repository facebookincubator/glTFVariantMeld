// Copyright (c) Facebook, Inc. and its affiliates. All Rights Reserved
//

use std::path::Path;

extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;

use serde_derive::{Deserialize, Serialize};

use crate::{Error, Tag, WorkAsset};

/// The Metadata struct & accessor methods
pub mod metadata;
pub use metadata::Metadata;

/// Compatibility methods for the WebAssembly build
pub mod wasm;

/// The primary API data structure.
///
/// The key property is a glTF asset in binary form (which will always implement
/// the `KHR_materials_variants` extension), along with various useful metadata for
/// the benefit of clients.
///
/// The key method melds one variational asset into another:
/// ```
///   extern crate assets;
///   use std::path::Path;
///   use gltf_variant_meld::{Tag, VariationalAsset};
///
///   let (matte_tag, shiny_tag) = (Tag::from("matte"), Tag::from("shiny"));
///   let pinecone_matte = VariationalAsset::from_file(
///      &Path::new(assets::ASSET_PINECONE_MATTE()),
///      Some(&matte_tag),
///   ).expect("Eek! Couldn't create matte pinecone VariationalAsset.");
///
///   let pinecone_shiny = VariationalAsset::from_file(
///      &Path::new(assets::ASSET_PINECONE_SHINY()),
///      Some(&shiny_tag),
///   ).expect("Eek! Couldn't create shiny pinecone VariationalAsset.");
///
///   let result = VariationalAsset::meld(
///     &pinecone_matte,
///     &pinecone_shiny
///   ).expect("Erk. Failed to meld two pinecones.");
///
///   assert!(result.metadata().tags().contains(&matte_tag));
///   assert!(result.metadata().tags().contains(&shiny_tag));
///   assert_eq!(result.metadata().tags().len(), 2);
///```
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct VariationalAsset {
    /// The generated glTF for this asset. Will always implement `KHR_materials_variants`
    /// and is always in binary (GLB) form.
    pub(crate) glb: Vec<u8>,

    /// The tag that stands in for any default material references in the asset glTF.
    pub(crate) default_tag: Tag,

    /// All the metadata generated for this asset.
    pub(crate) metadata: Metadata,
}

/// A summary of a mesh primitive's byte size requirements; currently textures only.
#[wasm_bindgen]
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct AssetSizes {
    /// Byte count for texture image data, in its raw encoded form.
    pub texture_bytes: usize,
}

// methods that wasm_bindgen can't cope with in their preferred form
impl VariationalAsset {
    /// Generates a new `VariationalAsset` from a glTF file.
    ///
    /// If the provided asset implements `KHR_materials_variants`, then `default_tag` must
    /// either be empty or match the default tag within the asset.
    ///
    /// If the asset doesn't implement `KHR_materials_variants`, then the argument
    /// `default_tag` must be non-empty. If it does, then `default_tag` must either match
    /// what's in the asset, or else be empty.
    pub fn from_file(file: &Path, default_tag: Option<&Tag>) -> Result<VariationalAsset, Error> {
        let loaded = WorkAsset::from_file(file, default_tag)?;
        loaded.export()
    }

    /// Generates a new `VariationalAsset` from a byte slice of glTF.
    ///
    /// If the provided asset implements `KHR_materials_variants`, then `default_tag` must
    /// either be empty or match the default tag within the asset.
    ///
    /// If the asset doesn't implement `KHR_materials_variants`, then the argument
    /// `default_tag` must be non-empty. If it does, then `default_tag` must either match
    /// what's in the asset, or else be empty.
    pub fn from_slice(
        gltf: &[u8],
        default_tag: Option<&Tag>,
        base_dir: Option<&Path>,
    ) -> Result<VariationalAsset, Error> {
        let loaded = WorkAsset::from_slice(gltf, default_tag, base_dir)?;
        loaded.export()
    }

    /// The generated glTF for this asset. Will always implement `KHR_materials_variants`
    /// and is always in binary (GLB) form.
    pub fn glb(&self) -> &[u8] {
        self.glb.as_slice()
    }

    /// The tag that stands in for any default material references in the asset glTF.
    pub fn default_tag(&self) -> &Tag {
        &self.default_tag
    }

    /// All the metadata generated for this asset.
    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    /// Melds one variational asset into another, combining material-switching tags
    /// on a per-mesh, per-primitive basis.
    ///
    /// Note that logically identical glTF objects may be bitwise quite different, e.g.
    /// because glTF array order is undefined, floating-point values are only equal to
    /// within some epsilon, the final global position of a vector can be the end
    /// result of very different transformations, and so on.
    ///
    /// Further, the whole point of this tool is to identify shared pieces of data
    /// between the two assets, keep only one, and redirect all references to it.
    ///
    pub fn meld<'a>(
        base: &'a VariationalAsset,
        other: &'a VariationalAsset,
    ) -> Result<VariationalAsset, Error> {
        let base = &WorkAsset::from_slice(base.glb(), Some(base.default_tag()), None)?;
        let other = &WorkAsset::from_slice(other.glb(), Some(other.default_tag()), None)?;

        let meld = WorkAsset::meld(base, other)?;
        meld.export()
    }
}

impl AssetSizes {
    /// Instantiate a new `AssetSizes` with the given texture byte count.
    pub fn new(texture_bytes: usize) -> AssetSizes {
        AssetSizes { texture_bytes }
    }

    /// Byte count for texture image data, in its raw encoded form.
    pub fn texture_bytes(&self) -> usize {
        self.texture_bytes
    }
}

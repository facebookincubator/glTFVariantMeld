// Copyright (c) Facebook, Inc. and its affiliates. All Rights Reserved
//

use std::collections::{HashMap, HashSet};

extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;

use serde_derive::{Deserialize, Serialize};
use serde_json::json;

use crate::{AssetSizes, Tag};

/// All the metadata generated for a variational asset.
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    /// The set of variational tags in this asset.
    pub(crate) tags: HashSet<Tag>,
    /// The sum byte size of **every** referenced texture in this asset.
    pub(crate) total_sizes: AssetSizes,
    /// The sum byte size of textures that are referenced depending on active variant tag.
    pub(crate) variational_sizes: AssetSizes,
    // The sum byte size of textures active under each variant tag specifically.
    pub(crate) per_tag_sizes: HashMap<Tag, AssetSizes>,
}

// methods that are already happily wasm_bind compliant
#[wasm_bindgen]
impl Metadata {
    /// The sum byte size of **every** referenced texture in this asset.
    pub fn total_sizes(&self) -> AssetSizes {
        self.total_sizes
    }

    /// The sum byte size of textures that are referenced depending on active variant tag.
    pub fn variational_sizes(&self) -> AssetSizes {
        self.variational_sizes
    }
}

// methods that wasm_bindgen can't cope with in their preferred form
impl Metadata {
    /// The set of variational tags in this asset.
    pub fn tags(&self) -> &HashSet<Tag> {
        &self.tags
    }

    /// The asset sizes associated with the given tag, if any.
    pub fn tag_sizes(&self, tag: &Tag) -> Option<&AssetSizes> {
        self.per_tag_sizes.get(tag)
    }
}

#[wasm_bindgen]
impl Metadata {
    /// WASM-friendly version of `tags()`; returns a JSON-encoded array of strings.
    pub fn wasm_tags(&self) -> String {
        json!(self.tags).to_string()
    }

    /// WASM-friendly version of `tags()`; returns a JSON-encoded map of tags to sizes.
    pub fn wasm_tag_sizes(&self) -> String {
        json!(self.per_tag_sizes).to_string()
    }
}

// Copyright (c) Facebook, Inc. and its affiliates. All Rights Reserved
//

extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;

use crate::{Metadata, Tag, VariationalAsset};

// simplified versions of methods for the benefit only of wasm_bind
#[wasm_bindgen]
impl VariationalAsset {
    /// WASM-friendly version of `from_slice`; remaps its errors as `JsValue`.
    pub fn wasm_from_slice(glb: &[u8], tag: Option<Tag>) -> Result<VariationalAsset, JsValue> {
        VariationalAsset::from_slice(glb, tag.as_ref(), None).map_err(JsValue::from)
    }

    /// WASM-friendly version of `meld``; remaps its errors as `JsValue`.
    pub fn wasm_meld(
        base: &VariationalAsset,
        melded: &VariationalAsset,
    ) -> Result<VariationalAsset, JsValue> {
        VariationalAsset::meld(base, melded).map_err(JsValue::from)
    }

    /// WASM-friendly version of `glb()`; returns an ownable `Vec<u8>` instead of a `&[u8]` slice.
    pub fn wasm_glb(&self) -> Vec<u8> {
        self.glb.to_owned()
    }

    /// WASM-friendly version of `default_tag()`; returns a clone of the tag
    pub fn wasm_default_tag(&self) -> Tag {
        self.default_tag.clone()
    }

    /// WASM-friendly version of `metadata()`; returns a clone of our metadata
    pub fn wasm_metadata(&self) -> Metadata {
        self.metadata.clone()
    }
}

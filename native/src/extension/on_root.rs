// Copyright (c) Facebook, Inc. and its affiliates. All Rights Reserved
//
use std::collections::{HashMap};
use serde_derive::{Deserialize, Serialize};

use gltf::json::Root;

use super::KHR_MATERIALS_VARIANTS;
use crate::{Result, Tag};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct FBMaterialVariantRootExtension {
    pub variants: Vec<FBMaterialVariantVariantEntry>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Default, Deserialize, Serialize)]
pub struct FBMaterialVariantVariantEntry {
    #[serde(default)]
    pub name: String,
}

/// Writes the root level variant lookup table containing object entries with each variant's
/// associated tag.
///
/// Please see [the `KHR_materials_variants`
/// spec](https://github.com/zellski/glTF/blob/ext/zell-fb-asset-variants/extensions/2.0/Khronos/KHR_materials_variants/README.md)
/// for further details.
pub fn write_root_variant_lookup_map(root: &mut Root, tags_in_use: &Vec<Tag>) -> Result<()> {
    // Transform list of tags into a list of object with object property name and tag
    let variant_entries: Vec<FBMaterialVariantVariantEntry> = tags_in_use
        .into_iter()
        .map(|tag| {
            FBMaterialVariantVariantEntry {
                name: tag.clone(),
            }
        })
        .collect();

    let root_extension = FBMaterialVariantRootExtension {
        variants: variant_entries,
    };

    let value = serde_json::to_string(&root_extension)
        .and_then(|s| serde_json::from_str(&s))
        .map_err(|e| {
            format!(
                "Failed to transform root extension {:#?}, with error: {}",
                root_extension, e,
            )
        })?;

    root
        .extensions
        .get_or_insert(Default::default())
        .others
        .insert(KHR_MATERIALS_VARIANTS.to_owned(), value);
    Ok(())
}

/// Extracts the variant lookup object from the root of the glTF file. This lookup is used to
/// translate Tags with indicies located on mesh primitives.
///
/// Please see [the `KHR_materials_variants`
/// spec](https://github.com/zellski/glTF/blob/ext/zell-fb-asset-variants/extensions/2.0/Khronos/KHR_materials_variants/README.md)
/// for further details.
pub fn get_variant_lookup(root: &Root) -> Result<HashMap<usize, Tag>> {
    match get_root_extension(&root)? {
        Some(extension) => {
            let mut lookup = HashMap::new();
            for (ix, variant) in extension.variants.iter().enumerate() {
                lookup.insert(ix, variant.name.to_owned());
            }
            Ok(lookup)
        }
        None => {
            Ok(HashMap::new())
        }
    }
}

fn get_root_extension(root: &Root) -> Result<Option<FBMaterialVariantRootExtension>> {
    if let Some(extensions) = &root.extensions {
        if let Some(ref boxed) = extensions.others.get(KHR_MATERIALS_VARIANTS) {
            let json_string = boxed.to_string();
            let parse: serde_json::Result<FBMaterialVariantRootExtension> =
                serde_json::from_str(&json_string);
            return match parse {
                Ok(parse) => {
                    Ok(Some(parse))
                }
                Err(e) => Err(format!(
                    "Bad JSON in KHR_materials_variants extension: {}; json = {}",
                    e.to_string(),
                    json_string,
                )),
            };
        }
    }
    Ok(None)
}

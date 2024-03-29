// Copyright (c) Facebook, Inc. and its affiliates. All Rights Reserved
//

use std::collections::{HashMap, HashSet};

use serde_derive::{Deserialize, Serialize};

use gltf::json::mesh::Primitive;

use super::KHR_MATERIALS_VARIANTS;
use crate::{Result, Tag};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct FBMaterialVariantPrimitiveExtension {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mappings: Vec<FBMaterialVariantPrimitiveEntry>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Default, Deserialize, Serialize)]
pub struct FBMaterialVariantPrimitiveEntry {
    #[serde(default)]
    pub material: u32,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub variants: Vec<u32>,
}

/// Write the `tag_to_ix` mapping to the `Primitive' in `KHR_materials_variants` form.
///
/// This method guarantees a deterministic ordering of the output.
///
/// Please see [the `KHR_materials_variants`
/// spec](https://github.com/zellski/glTF/blob/ext/zell-fb-asset-variants/extensions/2.0/Khronos/KHR_materials_variants/README.md)
/// for further details.
pub fn write_variant_map(primitive: &mut Primitive, tag_to_ix: &HashMap<Tag, usize>, variant_ix_lookup: &HashMap<usize, Tag>) -> Result<()> {
    if tag_to_ix.is_empty() {
        if let Some(extensions) = &mut primitive.extensions {
            extensions.others.remove(KHR_MATERIALS_VARIANTS);
        }
        return Ok(());
    }
    // invert the mapping tag->ix to a ix->set-of-tags one
    let mut ix_to_tags = HashMap::new();
    for (tag, &ix) in tag_to_ix {
        ix_to_tags
            .entry(ix)
            .or_insert(HashSet::new())
            .insert(tag.to_owned());
    }
    let mut mapping_entries: Vec<FBMaterialVariantPrimitiveEntry> = ix_to_tags
        .iter()
        .map(|(&ix, tags)| {
            let mut tag_vec: Vec<Tag> = tags.iter().cloned().collect();
            // order tags deterministically
            tag_vec.sort_unstable();

            let mut variants: Vec<u32> = tag_vec
                .iter()
                .map(|tag| {
                    let (&variant_ix, _) = variant_ix_lookup.iter().find(|(_k, v)| v == &tag).unwrap();
                    variant_ix as u32
                })
                .collect();
            variants.sort_unstable();

            FBMaterialVariantPrimitiveEntry {
                material: ix as u32,
                variants,
            }
        })
        .collect();
    // order entries deterministically
    mapping_entries.sort_unstable();
    // build structured extension data
    let new_extension = FBMaterialVariantPrimitiveExtension {
        mappings: mapping_entries,
    };
    // serialise to JSON string
    let value = serde_json::to_string(&new_extension)
        .and_then(|s| serde_json::from_str(&s))
        .map_err(|e| {
            format!(
                "Failed to transform primitive extension {:#?}, with error: {}",
                new_extension, e,
            )
        })?;

    // and done
    primitive
        .extensions
        .get_or_insert(Default::default())
        .others
        .insert(KHR_MATERIALS_VARIANTS.to_owned(), value);
    Ok(())
}

/// Parses and returns the `KHR_materials_variants` data on a primitive, if any.
///
/// Please see [the `KHR_materials_variants`
/// spec](https://github.com/zellski/glTF/blob/ext/zell-fb-asset-variants/extensions/2.0/Khronos/KHR_materials_variants/README.md)
/// for further details
pub fn extract_variant_map(primitive: &Primitive, variant_ix_lookup: &HashMap<usize, Tag>) -> Result<HashMap<Tag, usize>> {
    if let Some(extensions) = &primitive.extensions {
        if let Some(boxed) = extensions.others.get(KHR_MATERIALS_VARIANTS) {
            let json_string = &boxed.to_string();
            let parse: serde_json::Result<FBMaterialVariantPrimitiveExtension> =
                serde_json::from_str(json_string);
            return match parse {
                Ok(parse) => {
                    let mut result = HashMap::new();
                    for entry in parse.mappings {
                        for variant_ix in entry.variants {
                            let key = variant_ix as usize;
                            let variant_tag = &variant_ix_lookup[&key];
                            result.insert(variant_tag.to_owned(), entry.material as usize);
                        }
                    }
                    Ok(result)
                }
                Err(e) => Err(format!(
                    "Bad JSON in KHR_materials_variants extension: {}; json = {}",
                    e.to_string(),
                    json_string,
                )),
            };
        }
    }
    Ok(HashMap::new())
}

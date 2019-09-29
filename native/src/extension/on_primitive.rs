// Copyright (c) Facebook, Inc. and its affiliates. All Rights Reserved
//

use std::collections::{HashMap, HashSet};

use serde_derive::{Deserialize, Serialize};

use gltf::json::mesh::Primitive;

use crate::{Result, Tag};

#[allow(non_snake_case)]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct FBMaterialVariantPrimitiveExtras {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub FB_material_variants: Option<FBMaterialVariantPrimitiveExtension>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct FBMaterialVariantPrimitiveExtension {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mapping: Vec<FBMaterialVariantPrimitiveEntry>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Default, Deserialize, Serialize)]
pub struct FBMaterialVariantPrimitiveEntry {
    #[serde(default)]
    pub material: u32,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<Tag>,
}

/// Write the `tag_to_ix` mapping to the `Primitive' in `FB_material_variants` form.
///
/// This method guarantees a deterministic ordering of the output.
///
/// Please see [the `FB_material_variants`
/// spec](https://github.com/zellski/glTF/blob/ext/zell-fb-asset-variants/extensions/2.0/Vendor/FB_material_variants/README.md)
/// for further details.
pub fn write_variant_map(primitive: &mut Primitive, tag_to_ix: &HashMap<Tag, usize>) -> Result<()> {
    if tag_to_ix.is_empty() {
        primitive.extras = None;
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
            FBMaterialVariantPrimitiveEntry {
                material: ix as u32,
                tags: tags.iter().cloned().collect(),
            }
        })
        .collect();
    // order entries deterministically
    mapping_entries.sort_unstable();
    // build structured extension data
    let new_extras = FBMaterialVariantPrimitiveExtras {
        FB_material_variants: Some(FBMaterialVariantPrimitiveExtension {
            mapping: mapping_entries,
        }),
    };
    // serialise to JSON string
    let json_str = serde_json::to_string(&new_extras)
        .map_err(|e| format!("Failed to serialise extension data: {}", e))?;

    // deserialise JSON string to RawValue
    let raw = serde_json::from_str(&json_str)
        .map_err(|e| format!("Failed to build extension JSON: {}", e))?;

    // and done
    primitive.extras = Some(Box::from(raw));
    Ok(())
}

/// Parses and returns the `FB_material_variants` data on a primitive, if any.
///
/// Please see [the `FB_material_variants`
/// spec](https://github.com/zellski/glTF/blob/ext/zell-fb-asset-variants/extensions/2.0/Vendor/FB_material_variants/README.md)
/// for further details
pub fn extract_variant_map(primitive: &Primitive) -> Result<HashMap<Tag, usize>> {
    if let Some(boxed) = &primitive.extras {
        let json_string = &boxed.to_string();
        let parse: serde_json::Result<FBMaterialVariantPrimitiveExtras> =
            serde_json::from_str(json_string);
        match parse {
            Ok(parse) => {
                let mut result = HashMap::new();
                if let Some(extension) = parse.FB_material_variants {
                    for entry in extension.mapping {
                        for tag in entry.tags {
                            result.insert(tag, entry.material as usize);
                        }
                    }
                }
                Ok(result)
            }
            Err(e) => Err(format!(
                "Bad JSON in FB_material_variants extension: {}; json = {}",
                e.to_string(),
                json_string,
            )),
        }
    } else {
        Ok(HashMap::new())
    }
}

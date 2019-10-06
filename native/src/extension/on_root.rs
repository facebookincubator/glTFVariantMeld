// Copyright (c) Facebook, Inc. and its affiliates. All Rights Reserved
//

use serde_derive::{Deserialize, Serialize};

use gltf::json::Root;

use super::FB_MATERIAL_VARIANTS;
use crate::{Result, Tag};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct FBMaterialVariantRootExtension {
    pub default_tag: Tag,
}

/// Write the given `default_tag` into the JSON `Root` in `FB_material_variants` form.
///
/// Please see [the `FB_material_variants`
/// spec](https://github.com/zellski/glTF/blob/ext/zell-fb-asset-variants/extensions/2.0/Vendor/FB_material_variants/README.md)
/// for further details.
pub fn set_extension_tag(root: &mut Root, default_tag: &Tag) -> Result<()> {
    let extension = FBMaterialVariantRootExtension {
        default_tag: default_tag.to_owned(),
    };

    let value = serde_json::to_string(&extension)
        .and_then(|s| serde_json::from_str(&s))
        .map_err(|e| {
            format!(
                "Failed to transform root extension {:#?}, with error: {}",
                extension, e,
            )
        })?;

    root.extensions
        .get_or_insert(Default::default())
        .others
        .insert(FB_MATERIAL_VARIANTS.to_owned(), value);
    Ok(())
}

/// Parses and returns the default key in any `FB_material_variants` extension on the JSON `Root`.
///
/// Please see [the `FB_material_variants`
/// spec](https://github.com/zellski/glTF/blob/ext/zell-fb-asset-variants/extensions/2.0/Vendor/FB_material_variants/README.md)
/// for further details.
pub fn get_validated_extension_tag(root: &Root, default_tag: Option<&Tag>) -> Result<Tag> {
    let extension_tag = get_tag_from_extension(root)?;
    match default_tag {
        // no tag provided as argument: we require one from the extension data
        None => {
            if let Some(extension_tag) = extension_tag {
                Ok(extension_tag)
            } else {
                Err(format!("No default tag provided, and none found in asset."))
            }
        }
        Some(default_tag) => {
            // tag argument provided, we require extension to match, or be empty
            if let Some(extension_tag) = extension_tag {
                if extension_tag == *default_tag {
                    Ok(default_tag.to_owned())
                } else {
                    Err(format!(
                        "Provided tag ({}) does not match extension tag ({}).",
                        default_tag, extension_tag
                    ))
                }
            } else {
                Ok(default_tag.to_owned())
            }
        }
    }
}

fn get_tag_from_extension(root: &Root) -> Result<Option<Tag>> {
    Ok(get_root_extension(root)?.map(|ext| ext.default_tag))
}

fn get_root_extension(root: &Root) -> Result<Option<FBMaterialVariantRootExtension>> {
    if let Some(extensions) = &root.extensions {
        if let Some(ref boxed) = extensions.others.get(FB_MATERIAL_VARIANTS) {
            let json_string = boxed.to_string();
            let parse: serde_json::Result<FBMaterialVariantRootExtension> =
                serde_json::from_str(&json_string);
            return match parse {
                Ok(parse) => {
                    if parse.default_tag != "" {
                        Ok(Some(parse))
                    } else {
                        Err(format!(
                            "Missing default_tag in FB_material_variants root extension."
                        ))
                    }
                }
                Err(e) => Err(format!(
                    "Bad JSON in FB_material_variants extension: {}; json = {}",
                    e.to_string(),
                    json_string,
                )),
            };
        }
    }
    Ok(None)
}

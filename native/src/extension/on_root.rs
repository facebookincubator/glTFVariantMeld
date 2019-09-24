// Copyright (c) Facebook, Inc. and its affiliates. All Rights Reserved
//

use serde_derive::{Deserialize, Serialize};

use gltf::json::{extras::RawValue, Root};

use crate::{Result, Tag};

#[allow(non_snake_case)]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct FBMaterialVariantRootExtras {
    #[serde(default)]
    pub FB_material_variants: Option<FBMaterialVariantRootExtension>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct FBMaterialVariantRootExtension {
    pub default_tag: Tag,
}

/// Write the given `default_tag` into the JSON `Root` in `FB_material_variants` form.
///
/// TODO: Link the `FB_material_variants` spec here.
pub fn set_extension_tag(root: &mut Root, default_tag: &Tag) -> Result<()> {
    let new_extras = FBMaterialVariantRootExtras {
        FB_material_variants: Some(FBMaterialVariantRootExtension {
            default_tag: default_tag.to_owned(),
        }),
    };

    let json = serde_json::to_string_pretty(&new_extras);

    if let Ok(json) = json {
        if let Ok(raw) = RawValue::from_string(json) {
            root.extras = Some(Box::from(raw));
            return Ok(());
        }
    }
    Err(String::from(
        "Failed to set default tag on FB_material_variants extension.",
    ))
}

/// Parses and returns the default key in any `FB_material_variants` extension on the JSON `Root`.
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
    if let Some(boxed) = &root.extras {
        let json_string = &boxed.to_string();
        let parse: serde_json::Result<FBMaterialVariantRootExtras> =
            serde_json::from_str(json_string);
        match parse {
            Ok(parse) => {
                if let Some(extension) = parse.FB_material_variants {
                    if extension.default_tag != "" {
                        Ok(Some(extension))
                    } else {
                        Err(format!(
                            "Missing default_tag in FB_material_variants root extension."
                        ))
                    }
                } else {
                    Ok(None)
                }
            }
            Err(e) => Err(format!(
                "Bad JSON in FB_material_variants extension: {}; json = {}",
                e.to_string(),
                json_string,
            )),
        }
    } else {
        Ok(None)
    }
}

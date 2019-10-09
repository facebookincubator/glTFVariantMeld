// Copyright (c) Facebook, Inc. and its affiliates. All Rights Reserved
//

//! Core asset meld functionality.
//!
//! This module implements the core functionality of the whole library: iterating over the meshes of
//! one asset, locating its equivalent in another asset, and melding together the tagged material
//! uses of the two.

use spectral::prelude::*;

use gltf::json::{buffer::View, texture::Sampler, Image, Index, Material, Texture};

use crate::{Result, WorkAsset};

impl<'a> WorkAsset {
    /// Meld `WorkAsset` *other* into `WorkAsset` *base*, returning the result.
    ///
    /// We begin by cloning *base* then we selectively meld in glTF objects from *other*. Because
    /// glTF objects depend on one another recursively, melding a material will typically require
    /// melding textures, which requires melding images sources, and so on. For each such meld, the
    /// object may already exist in *base*, in which case we return its existing index reference, or
    /// it may be new, in which case we copy it over and return the newly created index.
    pub fn meld(base: &'a WorkAsset, other: &'a WorkAsset) -> Result<WorkAsset> {
        let mut result = base.clone();
        for (other_mesh_ix, other_mesh_key) in other.mesh_keys.iter().enumerate() {
            if let Some(base_mesh_ix) = base.mesh_ix(&other_mesh_key) {
                let base_primitives = &base.meshes()[base_mesh_ix].primitives;
                let other_primitives = &other.meshes()[other_mesh_ix].primitives;
                assert_that!(base_primitives.len()).is_equal_to(other_primitives.len());

                for primitive_ix in 0..other_primitives.len() {
                    let mut base_map = base.variant_mapping(base_mesh_ix, primitive_ix).clone();
                    let base_primitive = &base_primitives[primitive_ix];
                    if let Some(base_material) = base_primitive.material {
                        if !base_map.contains_key(&base.default_tag) {
                            base_map.insert(
                                base.default_tag.clone(),
                                base.material_keys[base_material.value()].to_owned(),
                            );
                        }
                    }

                    let mut other_map = other.variant_mapping(other_mesh_ix, primitive_ix).clone();

                    let base_print = base.mesh_primitive_fingerprints[base_mesh_ix][primitive_ix];
                    let other_primitive_ix = other
                        .find_almost_equal_fingerprint(other_mesh_ix, &base_print, None)
                        .ok_or(format!(
                            "Melded asset has no equivalent to base mesh {}, primitive {}.",
                            base_mesh_ix, primitive_ix
                        ))?;
                    if let Some(other_material) = other_primitives[other_primitive_ix].material {
                        if !other_map.contains_key(&other.default_tag) {
                            other_map.insert(
                                other.default_tag.clone(),
                                other.material_keys[other_material.value()].to_owned(),
                            );
                        }
                    }

                    let mut result_map = base_map.clone();

                    for other_tag in other_map.keys() {
                        if base_map.contains_key(other_tag) {
                            if base_map[other_tag] != other_map[other_tag] {
                                return Err(format!(
                                    "Base[{}/{}] vs Foreign[{}/{}]: Tag {} material mismatch!",
                                    base_mesh_ix,
                                    primitive_ix,
                                    other_mesh_ix,
                                    primitive_ix,
                                    other_tag,
                                ));
                            }
                            continue;
                        }
                        let other_material_key = &other_map[other_tag];

                        if let Some(other_material_ix) = other.material_ix(&other_material_key) {
                            let _new_material_ix = meld_in_material(
                                &mut result,
                                other,
                                Index::new(other_material_ix as u32),
                            );
                            result_map.insert(other_tag.clone(), other_material_key.clone());
                        } else {
                            return Err(format!(
                                "Other[{}/{}]: Material key {} not found!",
                                other_mesh_ix, primitive_ix, other_material_key
                            ));
                        }
                    }
                    result.mesh_primitive_variants[base_mesh_ix][primitive_ix] = result_map;
                }
            } else {
                return Err(format!(
                    "meldd mesh #{} has no corresponding mesh in base!",
                    other_mesh_ix
                ));
            }
        }
        Ok(result)
    }
}

// Note: the methods below are all on a very similar structure, and could be abstracted using e.g.
// macros, but in our experiments we didn't get much more readability, and the complexity increases
// quite a bit. We'll stick with a bit of copy-and-paste boilerplate for now.

/// Meld a glTF `image` (i.e. texture source) from from *other* into *base*.
fn meld_in_image(base: &mut WorkAsset, other: &WorkAsset, other_ix: Index<Image>) -> Index<Image> {
    let other_ix = other_ix.value();
    let key = &other.image_keys[other_ix];
    if let Some(ix) = base.image_ix(key) {
        return Index::new(ix as u32);
    }
    let mut new_object = other.images()[other_ix].clone();

    // meld logic
    assert_that!(new_object.buffer_view).is_some();
    new_object.buffer_view = Some(copy_byte_view(base, other, new_object.buffer_view.unwrap()));
    // end meld logic

    Index::new(base.push_image(new_object, key) as u32)
}

/// Meld a glTF `sampler` (texture filter/wrap configuration) from *other* into *base*.
fn meld_in_sampler(
    base: &mut WorkAsset,
    other: &WorkAsset,
    other_ix: Index<Sampler>,
) -> Index<Sampler> {
    let other_ix = other_ix.value();
    let key = &other.sampler_keys()[other_ix];
    if let Some(ix) = base.sampler_ix(key) {
        return Index::new(ix as u32);
    }
    let new_object = other.samplers()[other_ix].clone();

    // no current meld logic needed

    Index::new(base.push_sampler(new_object, key) as u32)
}

/// Meld a glTF `texture` (consisting of a `sampler` and an `image`) from *other* into *base*.
fn meld_in_texture(
    base: &mut WorkAsset,
    other: &WorkAsset,
    other_ix: Index<Texture>,
) -> Index<Texture> {
    let other_ix = other_ix.value();
    let key = &other.texture_keys()[other_ix];
    if let Some(ix) = base.texture_ix(key) {
        return Index::new(ix as u32);
    }
    let mut new_object = other.textures()[other_ix].clone();

    // meld logic
    new_object.source = meld_in_image(base, other, new_object.source);
    new_object.sampler = new_object.sampler.map(|s| meld_in_sampler(base, other, s));
    // end meld logic

    Index::new(base.push_texture(new_object, key) as u32)
}

/// Meld a glTF `material` from *other* into *base*.
///
/// For non-trivial materials, this typically requires melding in textures as well.
fn meld_in_material(
    base: &mut WorkAsset,
    other: &WorkAsset,
    other_ix: Index<Material>,
) -> Index<Material> {
    let other_ix = other_ix.value();
    let key = &other.material_keys[other_ix];
    if let Some(ix) = base.material_ix(key) {
        return Index::new(ix as u32);
    }
    let mut new_object = other.materials()[other_ix].clone();

    // laboriously hand-meld the five relevant textures
    if let Some(mut info) = new_object.normal_texture {
        info.index = meld_in_texture(base, other, info.index);
        new_object.normal_texture = Some(info);
    }
    if let Some(mut info) = new_object.occlusion_texture {
        info.index = meld_in_texture(base, other, info.index);
        new_object.occlusion_texture = Some(info);
    }
    if let Some(mut info) = new_object.emissive_texture {
        info.index = meld_in_texture(base, other, info.index);
        new_object.emissive_texture = Some(info);
    }
    if let Some(mut info) = new_object.pbr_metallic_roughness.base_color_texture {
        info.index = meld_in_texture(base, other, info.index);
        new_object.pbr_metallic_roughness.base_color_texture = Some(info);
    }
    if let Some(mut info) = new_object.pbr_metallic_roughness.metallic_roughness_texture {
        info.index = meld_in_texture(base, other, info.index);
        new_object.pbr_metallic_roughness.metallic_roughness_texture = Some(info);
    }
    // end meld logic

    Index::new(base.push_material(new_object, key) as u32)
}

fn copy_byte_view(
    base: &mut WorkAsset,
    foreign: &WorkAsset,
    foreign_ix: Index<View>,
) -> Index<View> {
    let view = foreign.buffer_view(foreign_ix.value());
    let slice = foreign.buffer_view_as_slice(&view);
    let new_ix = base.push_buffer_view_from_slice(slice) as u32;
    Index::new(new_ix)
}

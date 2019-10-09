// Copyright (c) Facebook, Inc. and its affiliates. All Rights Reserved
//

use std::collections::HashMap;

use gltf::json::{buffer::View, Image, Material, Mesh, Root};
use gltf::json::{texture::Sampler, Texture};

use crate::{Fingerprint, MeldKey, Result, Tag};

use crate::gltfext::add_buffer_view_from_slice;

pub mod construct;

pub mod export;

pub mod meld;

const EPS_FINGERPRINT: f64 = 1e-6;

/// The primary internal data structure, which enables and accelerates the melding operation.
///
/// The first half of the asset constitutes all the data needed to export fully variational glTF:
/// the source document and blob, a default tag, and the per- mesh, per-primitive mapping of tags to
/// materials.
///
/// The second half are meld keys for various glTF objects, which are used heavily in the melding
/// process.
///
#[derive(Clone, Debug)]
pub struct WorkAsset {
    /// The parsed JSON of the underlying asset.
    parse: Root,

    /// The binary content of the asset; textures, geometry, animation data, ...
    blob: Vec<u8>,

    /// The tag used to represent vanilla glTF's material references during a meld operation.
    ///
    /// See crate-level documentation for more exhaustive information on how this happens.
    default_tag: Tag,

    /// A glTF asset's geometry is laid out in a vector of meshes, each of which consists of a
    /// vector of mesh primitives. For each mesh primitive, the variational extension adds a
    /// mapping of variant tag -> material references. That data is stored in this field, and
    /// gets used during melding & during export.
    mesh_primitive_variants: Vec<Vec<HashMap<Tag, MeldKey>>>,

    /// A `MeldKey` for each `Image`; essentially a hash of the binary contents.
    image_keys: Vec<MeldKey>,
    /// A `MeldKey` for each `Material`; a straight-forward string expansion of its state.
    material_keys: Vec<MeldKey>,
    /// A `MeldKey` for each `Mesh`; a straight-forward string expansion of its state.
    mesh_keys: Vec<MeldKey>,
    /// A `MeldKey` for each `Sampler`; a straight-forward string expansion of its state.
    sampler_keys: Vec<MeldKey>,
    /// A `MeldKey` for each `Texture`; a straight-forward string expansion of its state.
    texture_keys: Vec<MeldKey>,

    mesh_primitive_fingerprints: Vec<Vec<f64>>,
}

impl WorkAsset {
    /// A slice view of the entire binary blob.
    pub fn blob_slice(&self) -> &[u8] {
        &self.blob.as_slice()
    }

    /// The mapping of `Tag` to material `MeldKey` for a given primitive of a given mesh.
    pub fn variant_mapping(&self, m_ix: usize, p_ix: usize) -> &HashMap<Tag, MeldKey> {
        let mesh_mappings = &self.mesh_primitive_variants[m_ix];
        let primitive_mapping = &mesh_mappings[p_ix];
        primitive_mapping
    }

    /// The slice of bytes that constitute the raw data of a given `Image`.
    pub fn read_image_bytes(&self, image: &Image) -> Result<&[u8]> {
        if let Some(view) = image.buffer_view {
            if let Some(view) = self.parse.get(view) {
                let offset = view.byte_offset.unwrap_or(0) as usize;
                let length = view.byte_length as usize;
                return Ok(&self.blob_slice()[offset..offset + length]);
            }
        }
        Err(format!("Internal error: Image with a URI field?!"))
    }

    /// A `View` representing the ix:th buffer view of the underlying asset.
    pub fn buffer_view(&self, ix: usize) -> &View {
        &self.parse.buffer_views[ix]
    }

    /// The slice of bytes underlying the given buffer view.
    pub fn buffer_view_as_slice(&self, view: &View) -> &[u8] {
        let start = view.byte_offset.unwrap_or(0) as usize;
        let end = start + view.byte_length as usize;
        &self.blob[start..end]
    }

    /// Clone our JSON data and blob, and create a `Gltf` wrapper around it.
    pub fn to_owned_gltf(&self) -> gltf::Gltf {
        gltf::Gltf {
            document: gltf::Document::from_json_without_validation(self.parse.clone()),
            blob: if self.blob.is_empty() {
                None
            } else {
                Some(self.blob.clone())
            },
        }
    }

    /// Search the `Primitives` of a `Mesh` non-exactly for a specific `Fingerprint`.
    pub fn find_almost_equal_fingerprint(
        &self,
        mesh_ix: usize,
        print: &Fingerprint,
        exclude_ix: Option<usize>,
    ) -> Option<usize> {
        let prints = &self.mesh_primitive_fingerprints[mesh_ix];
        for (primitive_ix, primitive_print) in prints.iter().enumerate() {
            if let Some(exclude_ix) = exclude_ix {
                if exclude_ix == primitive_ix {
                    println!("Excluding test for ix {}.", exclude_ix);
                    continue;
                }
            }
            if (primitive_print - print).abs() < EPS_FINGERPRINT {
                println!(
                    "Successfully tested {} against #{}: {}",
                    print, primitive_ix, primitive_print
                );
                return Some(primitive_ix);
            }
            println!(
                "No match testing {} against #{}: {}",
                print, primitive_ix, primitive_print
            );
        }
        return None;
    }

    /// Adds a new buffer view to the asset, returning its index.
    pub fn push_buffer_view_from_slice(&mut self, bytes: &[u8]) -> usize {
        add_buffer_view_from_slice(bytes, &mut self.parse.buffer_views, &mut self.blob).value()
    }
}

/// Provide accessors and mutators for images, materials, meshes, samplers and textures:
macro_rules! impl_accessors_and_mutators {
    ($ty:ty, $name:expr, $objects:ident, $keys:ident, $index_of_keys:ident, $push:ident) => {
        #[doc = " This asset's `"]
        #[doc = $name]
        #[doc = "` glTF objects."]
        pub fn $objects(&self) -> &Vec<$ty> {
            &self.parse.$objects
        }
        #[doc = " This asset's `"]
        #[doc = $name]
        #[doc = "` keys."]
        pub fn $keys(&self) -> &Vec<MeldKey> {
            &self.$keys
        }
        #[doc = " The index of the given `"]
        #[doc = $name]
        #[doc = "` key, if any."]
        pub fn $index_of_keys(&self, key: &MeldKey) -> Option<usize> {
            self.$keys().iter().position(|k| k == key)
        }
        #[doc = " Add a new `"]
        #[doc = $name]
        #[doc = "` glTF object, along with its computed key."]
        pub fn $push(&mut self, item: $ty, key: &MeldKey) -> usize
        where
            $ty: Clone,
        {
            let new_ix = self.parse.$objects.len();
            self.parse.$objects.push(item.clone());
            self.$keys.push(key.clone());
            new_ix
        }
    };

    ($ty:ty, $objects:ident, $keys:ident, $index_of_keys:ident, $push:ident) => {
        impl_accessors_and_mutators!($ty, stringify!($ty), $objects, $keys, $index_of_keys, $push);
    };
}

impl WorkAsset {
    impl_accessors_and_mutators!(Image, images, image_keys, image_ix, push_image);
    impl_accessors_and_mutators!(
        Material,
        materials,
        material_keys,
        material_ix,
        push_material
    );
    impl_accessors_and_mutators!(Mesh, meshes, mesh_keys, mesh_ix, push_mesh);
    impl_accessors_and_mutators!(Sampler, samplers, sampler_keys, sampler_ix, push_sampler);
    impl_accessors_and_mutators!(Texture, textures, texture_keys, texture_ix, push_texture);
}

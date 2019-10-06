// Copyright (c) Facebook, Inc. and its affiliates. All Rights Reserved
//

//! Code to generate a glTF asset from a `WorkAsset` instance.

use std::collections::{HashMap, HashSet};

use gltf::json::{Material, Root};

use crate::extension;
use crate::{AssetSizes, Metadata, Result, Tag, VariationalAsset};

use crate::glb::GlbChunk;

use super::*;

impl<'a> WorkAsset {
    /// Builds fully standalone variational glTF from this `WorkAsset`'s state.
    ///
    /// First, we put together the finished structured asset:
    /// - Clone the JSON in `WorkAsset.parse` as well as the `WorkAsset.blob` byte vector.
    /// - Write `WorkAsset.default_tag` into the root of the new JSON, using the
    ///   `FB_material_variants` extension.
    /// - Iterate over every mesh and mesh primitive in `WorkAsset.mesh_primitive_variants`,
    ///   and wherever there is a non-empty variational Tag->Material mapping, convert that to
    ///   `FB_material_variants` form and write it into the mesh primitive's JSON.
    ///
    /// Next, we count up all the metadata.
    ///
    /// Finally, the binary glTF (GLB) blob is generated, by serialising the glTF JSON into
    /// text form, and merging it with the binary blob (see `::crate::glb` for details.)
    pub fn export(&self) -> Result<VariationalAsset> {
        let (parse, blob, metadata) = self.prepare_for_export()?;
        let default_tag = self.default_tag.clone();
        let glb = self.build_glb_for_export(parse, blob.as_slice())?;

        Ok(VariationalAsset {
            glb,
            default_tag,
            metadata,
        })
    }

    fn prepare_for_export(&self) -> Result<(Root, Vec<u8>, Metadata)> {
        // clone our Root & blob for new export
        let mut root = self.parse.clone();
        let blob = self.blob.clone();

        // make note of the use of our glTF extension
        extension::install(&mut root);

        // then mutate the clone with our variational state
        self.export_extension_tag(&mut root)?;
        let metadata = self.export_variant_mapping(&mut root)?;

        Ok((root, blob, metadata))
    }

    // export our `default_tag` member into glTF form
    fn export_extension_tag(&self, root: &mut Root) -> Result<()> {
        extension::set_extension_tag(root, &self.default_tag)
    }

    // export our `mesh_primitive_variants` member into glTF form, by transforming the
    // tag->material_key mapping of each mesh/primitive to a tag->material_ix one, then
    // finally calling the glTF extension code to actually convert it to JSON.
    fn export_variant_mapping(&self, root: &mut Root) -> Result<Metadata> {
        let mut image_sizer = ImageSizes::new(&self);

        // for each mesh...
        for (m_ix, mesh) in root.meshes.iter_mut().enumerate() {
            // and for each of that mesh's primitives...
            for (p_ix, primitive) in mesh.primitives.iter_mut().enumerate() {
                // retrieve the mapping of tag->material_key
                let variant_mapping = self.variant_mapping(m_ix, p_ix);

                // prepare to build the mapping of tag->material_ix
                let mut tag_to_ix = HashMap::new();

                // loop over the (tag, key) entries in that mapping...
                for (tag, material_key) in variant_mapping.iter() {
                    // map the material key to a glTF material index...
                    if let Some(material_ix) = self.material_ix(material_key) {
                        if *tag == self.default_tag {
                            // there may be a mapping for the default tag, but if so the primitive
                            // must have a default material too, and they must match, and we do
                            // not keep or count it – it's treated elsewhere further down
                            if let Some(default_material_ix) = primitive.material {
                                if default_material_ix.value() == material_ix {
                                    continue;
                                }
                                return Err(format!(
                                    "Huh? Default material {} != variant map entry {} of default tag {}.",
                                    default_material_ix,
                                    material_ix, self.default_tag
                            ));
                            }
                            return Err(format!(
                                "Huh? No default material, but variant map entry {} of default tag {} ",
                                material_ix, self.default_tag
                            ));
                        }

                        // place it into the tag->material_ix mapping
                        tag_to_ix.insert(tag.to_owned(), material_ix);

                        // and update metadata
                        image_sizer.accumulate_material(material_ix, true);
                        image_sizer.accumulate_tagged_material(material_ix, tag);
                    } else {
                        return Err(format!("Huh? Non-existent meld key: {}", material_key));
                    }
                }

                // now handle the primitive's default material, if any
                if let Some(default_material_ix) = primitive.material {
                    let default_material_ix = default_material_ix.value();
                    let is_variational = !tag_to_ix.is_empty();

                    image_sizer.accumulate_material(default_material_ix, is_variational);
                    image_sizer.accumulate_tagged_material(default_material_ix, &self.default_tag);

                    if is_variational {
                        // only map the default tag if there's other tags already in the mapping
                        tag_to_ix.insert(self.default_tag.clone(), default_material_ix);
                    }
                };

                // finally write out the tag->material_ix mapping to glTF JSON
                extension::write_variant_map(primitive, &tag_to_ix)?;
            }
        }

        // ask metadata sizer to count up all the totals
        let (total_image_size, variational_image_size, per_tag_image_size) = image_sizer.count()?;
        // use it to create an authoritative set of all variational tags
        let tags: HashSet<Tag> = per_tag_image_size.keys().cloned().collect();

        // use it also to create the Tag->AssetSize mapping
        let per_tag_sizes: HashMap<Tag, AssetSizes> = tags
            .iter()
            .map(|tag| (tag.to_owned(), AssetSizes::new(per_tag_image_size[tag])))
            .collect();

        // finally construct & return the Metadata structure
        Ok(Metadata {
            tags,
            total_sizes: AssetSizes {
                texture_bytes: total_image_size,
            },
            variational_sizes: AssetSizes {
                texture_bytes: variational_image_size,
            },
            per_tag_sizes,
        })
    }

    // given a `Root` and a binary blob, create an actual GLB file
    fn build_glb_for_export(&self, export_parse: Root, export_blob: &[u8]) -> Result<Vec<u8>> {
        let json = export_parse.to_string_pretty();
        let json = json
            .map(|s| s.into_bytes())
            .map_err(|e| format!("JSON deserialisation error: {}", e))?;

        let json_chunk = GlbChunk::JSON(&json);
        let bin_chunk = if !export_blob.is_empty() {
            Some(GlbChunk::BIN(export_blob))
        } else {
            None
        };

        Ok(GlbChunk::to_bytes(json_chunk, bin_chunk)?)
    }
}

struct ImageSizes<'a> {
    asset: &'a WorkAsset,
    all_images: HashSet<usize>,
    variational_images: HashSet<usize>,
    per_tag_images: HashMap<Tag, HashSet<usize>>,
}

impl<'a> ImageSizes<'a> {
    fn new(asset: &'a WorkAsset) -> ImageSizes {
        ImageSizes {
            asset: asset,
            all_images: HashSet::new(),
            variational_images: HashSet::new(),
            per_tag_images: HashMap::new(),
        }
    }
    fn accumulate_material(&mut self, ix: usize, is_variational: bool) {
        let materials = self.asset.materials();
        accumulate_material_into_set(&materials[ix], &mut self.all_images);
        if is_variational {
            accumulate_material_into_set(&materials[ix], &mut self.variational_images);
        }
    }

    fn accumulate_tagged_material(&mut self, ix: usize, tag: &Tag) {
        let materials = self.asset.materials();
        let image_set = self
            .per_tag_images
            .entry(tag.to_owned())
            .or_insert(HashSet::new());
        accumulate_material_into_set(&materials[ix], image_set);
    }

    fn count(&self) -> Result<(usize, usize, HashMap<Tag, usize>)> {
        let mut all = 0;
        let mut variational = 0;
        let mut size_map = HashMap::new();

        for image_ix in &self.all_images {
            let size = image_size(&self.asset, *image_ix)?;
            size_map.insert(image_ix, size);

            all += size;
            if self.variational_images.contains(&image_ix) {
                variational += size;
            }
        }

        let tagged = {
            let mut result = HashMap::new();
            for (tag, image_ix_set) in &self.per_tag_images {
                result.insert(tag.clone(), {
                    let mut sum = 0;
                    for image_ix in image_ix_set {
                        sum += size_map.get(image_ix).ok_or_else(|| {
                            format!("Tag {} references unknown image ix {}!?", tag, image_ix)
                        })?;
                    }
                    sum
                });
            }
            result
        };

        Ok((all, variational, tagged))
    }
}

fn image_size(asset: &WorkAsset, image_ix: usize) -> Result<usize> {
    Ok(asset.read_image_bytes(&asset.images()[image_ix])?.len())
}

fn accumulate_material_into_set(material: &Material, image_set: &mut HashSet<usize>) {
    let pbr = &material.pbr_metallic_roughness;
    if let Some(ref tex_info) = pbr.base_color_texture {
        image_set.insert(tex_info.index.value());
    }
    if let Some(ref tex_info) = pbr.metallic_roughness_texture {
        image_set.insert(tex_info.index.value());
    }
    if let Some(ref tex_info) = material.normal_texture {
        image_set.insert(tex_info.index.value());
    }
    if let Some(ref tex_info) = material.occlusion_texture {
        image_set.insert(tex_info.index.value());
    }
    if let Some(ref tex_info) = material.emissive_texture {
        image_set.insert(tex_info.index.value());
    }
}

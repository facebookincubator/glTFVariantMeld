// Copyright (c) Facebook, Inc. and its affiliates. All Rights Reserved
//

//! Code to parse & index a glTF asset into `WorkAsset` format.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use spectral::prelude::*;

use gltf::json::{image::MimeType, mesh::Primitive, Mesh, Root};
use gltf::Gltf;

use crate::extension;
use crate::gltfext::{add_buffer_view_from_slice, set_root_buffer};
use crate::meld_keys::{build_fingerprint, HasKeyForVariants};
use crate::{Fingerprint, MeldKey, Result, Tag, WorkAsset};

impl WorkAsset {
    /// Constructs a `WorkAsset` from a file `Path` using `::from_slice`.
    pub fn from_file(file: &Path, default_tag: Option<&Tag>) -> Result<WorkAsset> {
        let slice = fs::read(file).map_err(|e| {
            format!(
                "Couldn't read asset file {}: {}",
                file.to_str().unwrap(),
                e.to_string()
            )
        })?;
        Self::from_slice(&slice, default_tag, file.parent())
    }

    /// Constructs a `WorkAsset` from a glTF byte slice, which can be text (JSON) or binary (GLB).
    ///
    /// We lean on `Gltf::from_slice()` to parse the contents, yielding a `Document`
    /// (which wraps the JSON component we're really after) and a byte blob, which we will
    /// read from and may add to during other operations on this asset.
    ///
    /// See constructor `new()` for details on how the rest of `WorkAsset` is built.
    pub fn from_slice(
        gltf: &[u8],
        default_tag: Option<&Tag>,
        file_base: Option<&Path>,
    ) -> Result<WorkAsset> {
        let result = Gltf::from_slice(gltf).or_else(|e| {
            Err(format!(
                "Parse error in VariationalAsset glTF: {}",
                e.to_string()
            ))
        })?;

        // break the `Gltf` object into a `Root` and a byte blob here
        let parse = result.document.into_json();
        let blob = if let Some(blob) = result.blob {
            assert_that!(parse.buffers.len()).is_equal_to(1);
            assert_that!(parse.buffers[0].byte_length as usize)
                .is_less_than_or_equal_to(blob.len());
            blob
        } else {
            vec![]
        };

        Self::new(parse, blob, default_tag, file_base)
    }

    /// Constructs a `WorkAsset` given a JSON `Root`, a byte blob, default tag & file base.
    ///
    /// First, any filesystem references within the glTF are converted to binary references, by
    /// resolving paths, reading files, and appending them to the blob & as `BufferView` objects in
    /// the JSON. After this step, the asset is entirely self-contained, and the `file_base`
    /// argument is no longer used.
    ///
    /// Next, we validate/retrieve any default tag embedded using the `FB_material_variants`
    /// extension. This tag must match the `default_tag` provided as argument, if any, and if none
    /// was provided it must exist in the asset. This ensure `WorkAsset.default_tag` always exists
    /// and makes sense.
    ///
    /// Then, we construct `MeldKey` strings for every glTF object we track – `Image`, `Sampler`,
    /// `Texture`, `Material` and `Mesh`. Please consult the `::meld_keys` module for details on
    /// meld keys.
    ///
    /// Finally, each mesh and mesh primitive is inspected, and any `FB_material_variants` data is
    /// parsed and converted to a Tag->MeldKey mapping, filling in `mesh_primitive_variants` and
    /// completing the `WorkAsset` construction.
    pub fn new(
        mut parse: Root,
        mut blob: Vec<u8>,
        default_tag: Option<&Tag>,
        file_base: Option<&Path>,
    ) -> Result<WorkAsset> {
        Self::transform_parse(&mut parse, &mut blob, file_base)?;

        let default_tag = extension::get_validated_extension_tag(&parse, default_tag)?;

        let mut asset = WorkAsset {
            parse,
            blob,
            default_tag,
            mesh_primitive_variants: vec![],

            image_keys: vec![],
            material_keys: vec![],
            mesh_keys: vec![],
            sampler_keys: vec![],
            texture_keys: vec![],

            mesh_primitive_fingerprints: vec![],
        };

        // there is a strict dependency order here which must be observed
        asset.image_keys = asset.build_meld_keys(&asset.parse.images)?;
        asset.sampler_keys = asset.build_meld_keys(&asset.parse.samplers)?;
        asset.texture_keys = asset.build_meld_keys(&asset.parse.textures)?;
        asset.material_keys = asset.build_meld_keys(&asset.parse.materials)?;
        asset.mesh_keys = asset.build_meld_keys(&asset.parse.meshes)?;
        asset.mesh_primitive_fingerprints = asset.build_fingerprints()?;

        asset.ensure_unique_mesh_keys()?;
        asset.ensure_uniqueish_fingerprints()?;

        let mesh_primitive_variants = asset.map_variants()?;
        asset.mesh_primitive_variants = mesh_primitive_variants;

        Ok(asset)
    }

    fn build_meld_keys<T: HasKeyForVariants>(&self, objects: &Vec<T>) -> Result<Vec<MeldKey>> {
        let vec_of_results: Vec<Result<MeldKey>> = objects
            .iter()
            .map(|o| o.build_meld_key(self))
            .to_owned()
            .collect();
        vec_of_results.into_iter().collect()
    }

    fn build_fingerprints(&self) -> Result<Vec<Vec<Fingerprint>>> {
        let gltf = self.to_owned_gltf();

        let mut result = vec![];
        for mesh in gltf.meshes() {
            let mut fingerprints = vec![];
            for primitive in mesh.primitives() {
                fingerprints.push(build_fingerprint(&primitive, &self.blob)?);
            }
            result.push(fingerprints);
        }
        Ok(result)
    }

    fn ensure_unique_mesh_keys(&self) -> Result<()> {
        let mut seen = HashSet::new();
        let mut dups = HashSet::new();
        for mesh_key in &self.mesh_keys {
            if seen.contains(mesh_key) {
                dups.insert(mesh_key);
            }
            seen.insert(mesh_key);
        }
        if !dups.is_empty() {
            Err(format!("Aii, non-unique meld keys: {:#?}", dups))
        } else {
            Ok(())
        }
    }

    fn ensure_uniqueish_fingerprints(&self) -> Result<()> {
        for (mesh_ix, fingerprints) in self.mesh_primitive_fingerprints.iter().enumerate() {
            for (primitive_ix, fingerprint) in fingerprints.iter().enumerate() {
                if let Some(other_print) =
                    self.find_almost_equal_fingerprint(mesh_ix, fingerprint, Some(primitive_ix))
                {
                    return Err(format!(
                        "Can't cope with primitives {} and {} of mesh {} being identical.",
                        primitive_ix, other_print, mesh_ix
                    ));
                }
            }
        }
        Ok(())
    }

    fn map_variants(&self) -> Result<Vec<Vec<HashMap<Tag, MeldKey>>>> {
        let map_material = |(tag, ix): (&MeldKey, &usize)| -> Result<(Tag, MeldKey)> {
            Ok((tag.to_string(), self.material_keys[*ix].to_owned()))
        };
        let map_primitive = |p: &Primitive| -> Result<HashMap<Tag, MeldKey>> {
            let variant_map = extension::extract_variant_map(p)?;
            variant_map.iter().map(map_material).collect()
        };
        let map_mesh = |m: &Mesh| -> Result<Vec<HashMap<Tag, MeldKey>>> {
            m.primitives.iter().map(map_primitive).collect()
        };
        self.parse.meshes.iter().map(map_mesh).collect()
    }

    // ensure the glTF is in the state that WorkAsset expects
    fn transform_parse(
        root: &mut Root,
        blob: &mut Vec<u8>,
        file_base: Option<&Path>,
    ) -> Result<()> {
        // load from URI any non-GLB buffers
        Self::transform_buffers(root, blob, file_base)?;
        // load from URI any images not already embedded
        Self::transform_images(root, blob, file_base)?;
        Ok(())
    }

    // resolve any buffers in the asset that reference URIs, read those files
    // and append them to the binary blob, and finally replace the entire buffer
    // vector with our single BIN buffer entry
    fn transform_buffers(
        root: &mut Root,
        blob: &mut Vec<u8>,
        file_base: Option<&Path>,
    ) -> Result<()> {
        assert_that!(blob.len() % 4).is_equal_to(0);

        for buffer in &mut root.buffers {
            if let Some(uri) = &buffer.uri {
                let mut buffer_bytes = Self::read_from_uri(uri, file_base)?;
                blob.append(&mut buffer_bytes);
                while (blob.len() % 4) != 0 {
                    blob.push(0x00);
                }
            }
        }

        set_root_buffer(blob, &mut root.buffers);

        Ok(())
    }

    // resolve any images in the asset that reference URIs, read those files and create
    // buffer_views for them and add them + buffer views to the asset.
    fn transform_images(
        root: &mut Root,
        blob: &mut Vec<u8>,
        file_base: Option<&Path>,
    ) -> Result<()> {
        let images = &mut root.images;
        let buffer_views = &mut root.buffer_views;

        for img in images {
            if img.buffer_view.is_none() {
                if let Some(uri) = &img.uri {
                    let image_bytes = Self::read_from_uri(uri, file_base)?;
                    let view_ix =
                        add_buffer_view_from_slice(image_bytes.as_slice(), buffer_views, blob);

                    img.buffer_view = Some(view_ix);
                    img.mime_type = Some(Self::guess_mime_type(uri)?);
                    img.uri = None;
                }
            }
        }
        Ok(())
    }

    fn guess_mime_type(uri: &String) -> Result<MimeType> {
        if let Some(extension) = Path::new(uri).extension() {
            match &extension.to_str().unwrap().to_ascii_lowercase()[..] {
                "jpg" | "jpeg" => {
                    return Ok(MimeType("image/jpeg".to_string()));
                }
                "png" => {
                    return Ok(MimeType("image/png".to_string()));
                }
                _ => {}
            }
        };
        Err(format!("Can't guess mime type of URI: {}", uri))
    }

    fn read_from_uri(uri: &str, file_base: Option<&Path>) -> Result<Vec<u8>> {
        // this is very temporary, lifted lifted from gltf::import.rs
        let path = if uri.contains(":") {
            if uri.starts_with("file://") {
                &uri["file://".len()..]
            } else if uri.starts_with("file:") {
                &uri["file:".len()..]
            } else {
                panic!("Can only handle file:// URIs yet.");
            }
        } else {
            &uri[..]
        };
        let mut path = PathBuf::from(path);
        if path.is_relative() {
            if let Some(file_base) = file_base {
                path = file_base.join(path);
            }
        }
        Ok(fs::read(path.as_path())
            .map_err(|e| format!("Error reading file {}: {}", path.display(), e.to_string()))?)
    }
}

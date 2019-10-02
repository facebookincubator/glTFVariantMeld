// Copyright (c) Facebook, Inc. and its affiliates. All Rights Reserved
//

use sha1::Sha1;

use gltf::json::texture;
use gltf::json::{material::NormalTexture, material::OcclusionTexture};
use gltf::json::{texture::Sampler, Image, Index, Material, Mesh, Texture};

use crate::{MeldKey, Result, WorkAsset};

/// A trait implemented on glTF objects for which we need a `MeldKey`.
///
/// Please consult [the `MeldKey` documentation](../meld_keys/type.MeldKey.html) for an overview.
pub trait HasKeyForVariants {
    /// Computes and returns the `MeldKey` for this object.
    ///
    /// See individual implementations for details.
    fn build_meld_key(&self, work_asset: &WorkAsset) -> Result<MeldKey>;
}

impl HasKeyForVariants for Image {
    /// The `MeldKey` of an `Image` is a stringified SHA1-hash of the underlying bytes.
    ///
    /// Example: "`daf12297c5c549fa199b85adbe77d626edc93184`"
    fn build_meld_key(&self, work_asset: &WorkAsset) -> Result<MeldKey> {
        let image_bytes = work_asset.read_image_bytes(self)?;
        Ok(Sha1::from(image_bytes).digest().to_string())
    }
}

impl HasKeyForVariants for Texture {
    /// The `MeldKey` of an `Texture` combines a `Sampler` and an `Image` `MeldKey`.
    ///
    /// Example: "`[sampler=,source=daf12297c5c549fa199b85adbe77d626edc93184]`"
    fn build_meld_key(&self, work_asset: &WorkAsset) -> Result<MeldKey> {
        Ok(format!(
            "[sampler={},source={}]",
            key_or_empty(work_asset.sampler_keys(), self.sampler),
            key(work_asset.image_keys(), self.source),
        ))
    }
}

impl HasKeyForVariants for Sampler {
    /// The `MeldKey` of a `Sampler` is a stringification of simple JSON attributes.
    ///
    /// Example: "`[mag_filter=None,min_filter=None,wrap_s=Repeat,wrap_t=Repeat]`"
    fn build_meld_key(&self, _work_asset: &WorkAsset) -> Result<MeldKey> {
        Ok(format!(
            "[mag_filter={:?},min_filter={:?},wrap_s={:?},wrap_t={:?}]",
            self.mag_filter, self.min_filter, self.wrap_s, self.wrap_t
        ))
    }
}

impl HasKeyForVariants for Material {
    /// The `MeldKey` of a `Material` combines `Texture` keys with its own many JSON attributes.
    ///
    /// Example: "`[[pbr=[bcf=[1.0, 1.0, 1.0, 1.0], bct=[tc=0,src=[sampler=,source=49ff16b74ed7beabc95d49ef8a0f7615db949851]], mf=0.4, rf=0.6, mrt=[]], nt=[], ot=[], et=[], ef=[0.0, 0.0, 0.0], am=Opaque, ac=0.5, ds=false]`"
    fn build_meld_key(&self, work_asset: &WorkAsset) -> Result<MeldKey> {
        let pbr = &self.pbr_metallic_roughness;
        Ok(format!(
            "[[pbr=[bcf={:?}, bct={}, mf={:?}, rf={:?}, mrt={}], nt={}, ot={}, et={}, ef={:?}, am={:?}, ac={:?}, ds={}]",
            pbr.base_color_factor,
            key_for_texinfo(work_asset, &pbr.base_color_texture),
            pbr.metallic_factor,
            pbr.roughness_factor,
            key_for_texinfo(work_asset, &pbr.metallic_roughness_texture),
            key_for_normal_texinfo(work_asset, &self.normal_texture),
            key_for_occlusion_texinfo(work_asset, &self.occlusion_texture),
            key_for_texinfo(work_asset, &self.emissive_texture),
            self.emissive_factor,
            self.alpha_mode,
            self.alpha_cutoff,
            self.double_sided,
        ))
    }
}

impl HasKeyForVariants for Mesh {
    /// The `MeldKey` of a `Mesh` is simply its name. This is probably a temporary solution.
    ///
    /// Example: "`polySurface12`"
    ///
    /// Note: It'd be very, very convenient if we can match up meshes by name, because comparing
    /// them numerically is kind of a nightmare of fuzzy computational geometry. The question is if
    /// the tool can require users to control the glTF level name to the extend necessary.
    fn build_meld_key(&self, _work_asset: &WorkAsset) -> Result<MeldKey> {
        self.name
            .as_ref()
            .map(String::from)
            .ok_or_else(|| format!("Mesh with no name! Eee."))
    }
}

fn key_for_texinfo(work_asset: &WorkAsset, texinfo: &Option<texture::Info>) -> MeldKey {
    if let Some(texinfo) = &texinfo {
        format!(
            "[tc={},src={}]",
            texinfo.tex_coord,
            key(work_asset.texture_keys(), texinfo.index),
        )
    } else {
        String::from("[]")
    }
}

fn key_for_normal_texinfo(work_asset: &WorkAsset, texinfo: &Option<NormalTexture>) -> MeldKey {
    if let Some(texinfo) = &texinfo {
        format!(
            "[s={},tc={},src={}]",
            texinfo.scale,
            texinfo.tex_coord,
            key(work_asset.texture_keys(), texinfo.index),
        )
    } else {
        String::from("[]")
    }
}

fn key_for_occlusion_texinfo(
    work_asset: &WorkAsset,
    texinfo: &Option<OcclusionTexture>,
) -> MeldKey {
    if let Some(texinfo) = &texinfo {
        format!(
            "[s={:?},tc={},src={}]",
            texinfo.strength,
            texinfo.tex_coord,
            key(work_asset.texture_keys(), texinfo.index),
        )
    } else {
        String::from("[]")
    }
}

fn key_or_empty<T>(keys: &Vec<MeldKey>, ix: Option<Index<T>>) -> MeldKey {
    match ix {
        Some(ix) => key(keys, ix),
        None => String::new(),
    }
}

fn key<T>(keys: &Vec<MeldKey>, ix: Index<T>) -> MeldKey {
    keys[ix.value()].to_owned()
}

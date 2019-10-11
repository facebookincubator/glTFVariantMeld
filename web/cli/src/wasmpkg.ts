// Copyright (c) Facebook, Inc. and its affiliates. All Rights Reserved
//

import { VariationalAsset } from "../node_modules/glTFVariantMeld/gltf_variant_meld";

export type glTFVariantMeld = typeof import("../node_modules/glTFVariantMeld/gltf_variant_meld.js");

export async function runWithVariantMeld(callback: (v: glTFVariantMeld) => void) {
  let module = await import("../node_modules/glTFVariantMeld/gltf_variant_meld.js");
  callback(module);
}

// Copyright (c) Facebook, Inc. and its affiliates. All Rights Reserved
//

import { VariationalAsset } from "../node_modules/@zellski/variationator/variationator";

export type Variationator = typeof import("../node_modules/@zellski/variationator/variationator.js");

export async function runWithVariationator(callback: (v: Variationator) => void) {
  let module = await import("../node_modules/@zellski/variationator/variationator.js");
  callback(module);
}

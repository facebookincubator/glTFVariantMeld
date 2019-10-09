// Copyright (c) Facebook, Inc. and its affiliates. All Rights Reserved
//

mod key_trait;
pub use key_trait::HasKeyForVariants;

mod fingerprints;
pub use fingerprints::build_fingerprint;

/// A short string that uniquely identifies all glTF objects other than `Mesh` `Primitives`.
pub type MeldKey = String;

/// A floating-point number that identifies logically identical `Mesh` `Primitives`.
///
/// Most glTF objects are given a simple, unique key as part of the `MeldKey` mechanism.
/// For geometry, things are trickier. To begin with, neither the order of triangles (indices)
/// nor vectors are important, so any comparison must be order-agnostic. Worse, floating-point
/// calculations are inexact, and so identity there must be of the ||x - x'|| < ε type.
pub type Fingerprint = f64;

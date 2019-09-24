// Copyright (c) Facebook, Inc. and its affiliates. All Rights Reserved
//

mod key_trait;
pub use key_trait::HasKeyForVariants;

/// A short string that uniquely identifies certain glTF objects.
///
/// Two logically identical objects can be bitwise quite different between different glTF
/// exports, different versions of the exporter, or even different runs with the same exporter.
///
/// For example:
/// - The order of glTF object arrays is unimportant, and may vary freely between runs.
/// - Scene graphs can be constructed in countless ways while representing the same structure.
/// - Neither mesh triangles nor vertices are strictly ordered, so identity is non-trivial.
/// - Floating-point computation is inexact, so comparisons should be fuzzy, à la `||Δv|| < ε`.
///
/// NOTE: In practice, we currently compute these keys in a very straight-forward fashion. The
/// one for a `Mesh`, for example, is simply its name. This likely won't suffice in the long run.
pub type MeldKey = String;

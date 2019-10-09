// Copyright (c) Facebook, Inc. and its affiliates. All Rights Reserved
//

use gltf::{mesh::Primitive, Buffer};

use spectral::prelude::*;

use crate::{Fingerprint, Result};

/// Computes a `Fingerprint` from a `Primitive`.
///
/// A fingerprint needs to be independent of triangle order and vertex order, and obviously it
/// should be non-trivially different from the fingerprint of some other `Primitive`. This isn't
/// as obvious as it seems: for example, if we simply took the geometric average of positions,
/// all shapes that are symmetric around origin, regardless of scale, would be identical.
///
/// We look at vertex positions and vertex colours, and simply add them up, with an added
/// skew to the Y and Z dimensions, to break symmetries.
///
/// More complexity could be added here, if warranted.
pub fn build_fingerprint(primitive: &Primitive, blob: &[u8]) -> Result<Fingerprint> {
    let buf_to_blob = |buf: Buffer| {
        assert_that(&buf.index()).is_equal_to(0);
        if blob.is_empty() {
            None
        } else {
            Some(blob)
        }
    };

    let reader = primitive.reader(buf_to_blob);

    let positions: Vec<[f32; 3]> = reader
        .read_positions()
        .ok_or(format!("Primitive lacks position data!"))?
        .collect();

    let indices: Vec<u32> = reader
        .read_indices()
        .ok_or(format!("Primitive lacks indices!"))?
        .into_u32()
        .collect();

    let count = indices.len() as f64;
    println!("Index count: {}", count);

    let mut cumulative_fingerprint = {
        let mut print: f64 = 0.0;
        for &ix in &indices {
            print += vec3_to_print(positions[ix as usize]) / count;
        }
        print
    };

    if let Some(colors) = reader.read_colors(0) {
        let colors: Vec<[f32; 4]> = colors.into_rgba_f32().collect();

        cumulative_fingerprint += {
            let mut print: f64 = 0.0;
            for &ix in &indices {
                print += vec4_to_print(colors[ix as usize]) / count;
            }
            print
        }
    }

    Ok(cumulative_fingerprint)
}

fn vec3_to_print(vec: [f32; 3]) -> f64 {
    // arbitrary symmetry-breaking shear
    (vec[0] + 1.3 * vec[1] + 1.7 * vec[2]) as f64
}

fn vec4_to_print(vec: [f32; 4]) -> f64 {
    // arbitrary symmetry-breaking shear
    (vec[0] + 1.1 * vec[1] + 1.3 * vec[2] + 1.5 * vec[3]) as f64
}

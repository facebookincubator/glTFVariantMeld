// Copyright (c) Facebook, Inc. and its affiliates. All Rights Reserved
//

//! Utilities for building binary glTF (GLB) files.

use crate::Result;

use GlbChunk::{BIN, JSON};

const GLB_VERSION: u32 = 2;
const GLB_MAGIC: [u8; 4] = [b'g', b'l', b'T', b'F'];

/// GLB 2.0 holds one JSON chunk followed by an optional BIN chunk.
pub enum GlbChunk<'a> {
    /// A byte slice of valid JSON conforming to the [glTF schema].
    ///
    /// [glTF schema]: https://github.com/KhronosGroup/glTF/tree/master/specification/2.0
    JSON(&'a [u8]),
    /// An binary blob, destined to become the data underlying a glTF buffer.
    BIN(&'a [u8]),
}

impl<'a> GlbChunk<'a> {
    fn magic(&self) -> u32 {
        match *self {
            JSON(_) => 0x4E4F534A,
            BIN(_) => 0x004E4942,
        }
    }
    fn bytes(&self) -> &[u8] {
        match *self {
            JSON(bytes) => bytes,
            BIN(bytes) => bytes,
        }
    }

    /// Serialised JSON & optional BIN chunks binary glTF, i.e. GLB 2.0.
    pub fn to_bytes(json_chunk: Self, bin_chunk: Option<Self>) -> Result<Vec<u8>> {
        // create the initial header
        let mut glb_bytes = vec![];
        glb_bytes.extend_from_slice(&GLB_MAGIC);
        glb_bytes.extend_from_slice(&(GLB_VERSION as u32).to_le_bytes());
        glb_bytes.extend_from_slice(&(0 as u32).to_le_bytes()); // fill in later

        let mut append_chunk = |chunk: Self| {
            let mut chunk_bytes = chunk.bytes().to_vec();
            if chunk_bytes.len() > 0 {
                while (chunk_bytes.len() % 4) != 0 {
                    chunk_bytes.push(if let JSON(_) = chunk { b' ' } else { 0x00 });
                }
                glb_bytes.extend_from_slice(&(chunk_bytes.len() as u32).to_le_bytes());
                glb_bytes.extend_from_slice(&(chunk.magic() as u32).to_le_bytes());
                glb_bytes.extend_from_slice(&chunk_bytes);
            }
        };

        if let JSON(_) = json_chunk {
            append_chunk(json_chunk);
        } else {
            return Err(format!("First GLB chunk must be of type JSON."));
        }
        if let Some(bin_chunk) = bin_chunk {
            if let BIN(_) = bin_chunk {
                append_chunk(bin_chunk);
            } else {
                return Err(format!("Second GLB chunk must be of type BIN, or None."));
            }
        }

        let glb_len_bytes = &(glb_bytes.len() as u32).to_le_bytes();
        for i in 0..3 {
            glb_bytes[0x08 + i] = glb_len_bytes[i];
        }
        Ok(glb_bytes)
    }
}

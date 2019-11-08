// Copyright (c) Facebook, Inc. and its affiliates. All Rights Reserved
//

//! Utility functions that extend the functionality of the `gltf` crate(s) for our needs.

use gltf::json::{buffer::View, Buffer, Index};

use crate::Result;

/// Returns the underlying byte slice of the given buffer view.
pub fn get_slice_from_buffer_view<'a>(view: &'a View, blob: &'a Vec<u8>) -> Result<&'a [u8]> {
    let start = view.byte_offset.unwrap_or(0) as usize;
    let end = start + view.byte_length as usize;
    (&blob.get(start..end)).ok_or_else(|| {
        format!(
            "Slice [{}..{}] out of range for buffer view of length {}.",
            start, end, view.byte_length
        )
    })
}

/// Adds a byte slice to the given blob, creates & pushes a buffer view onto the given vector.
///
/// This method ensures the byte slice ends up at a 4-byte-aligned position in the blob.
pub fn add_buffer_view_from_slice(
    bytes: &[u8],
    buffer_views: &mut Vec<View>,
    blob: &mut Vec<u8>,
) -> Index<View> {
    while (blob.len() % 4) != 0 {
        blob.push(0x00);
    }

    let view_ix = buffer_views.len();
    let view = View {
        buffer: Index::new(0),
        byte_length: bytes.len() as u32,
        byte_offset: Some(blob.len() as u32),
        byte_stride: None,
        name: None,
        target: None,
        extensions: None,
        extras: None,
    };
    buffer_views.push(view);

    blob.extend_from_slice(bytes);

    Index::new(view_ix as u32)
}

/// Replaces any contents of the provided buffer vector with a single one, holding the given blob.
pub fn set_root_buffer(blob: &[u8], buffers: &mut Vec<Buffer>) {
    buffers.clear();
    if !blob.is_empty() {
        buffers.push(Buffer {
            byte_length: blob.len() as u32,
            uri: None,
            name: None,
            extensions: None,
            extras: None,
        });
    }
}

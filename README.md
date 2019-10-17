# glTFVariantMeld

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![CircleCI](https://circleci.com/gh/facebookincubator/glTFVariantMeld/tree/master.svg?style=svg&circle-token=444333da241c0fc99a7ac8f786129f3bce774b43)](https://circleci.com/gh/facebookincubator/glTFVariantMeld/tree/master)
[![Actions Status](https://github.com/facebookincubator/glTFVariantMeld/workflows/Rust/badge.svg)](https://github.com/facebookincubator/glTFVariantMeld/actions)

## Description

This tool melds multiple glTF assets, each representing a different _variant_ of a model, into a single, compact format, implemented as a glTF extension.

A canonical use case is a retail product that's available in a range of colour combinations, with an
application that lets a prospective customer switch between these different variants with minimal
latency.

We're making this internal tool publicly available with the hope of helping the glTF
ecosystem come together around a common, open format. In this prerelease version, the tool
produces files with the vendor extension
[`FB_material_variants`](https://github.com/KhronosGroup/glTF/pull/1681). We are hopeful
that the glTF community will find speedy consensus around a ratified extension.

In this prerelease version, the tool produces files with the vendor extension [`FB_material_variants`](https://github.com/KhronosGroup/glTF/blob/f0ab429b4260cfa91925bcf5044624968773902c/extensions/2.0/Vendor/FB_material_variants/README.md). We are hopeful that the glTF community will find speedy consensus around a multi-vendor extension.

Our aspirational roadmap includes the development of a web app which would leverage
WebAssembly to run entirely in the browser. There will also be a native CLI.

**Assistance is always welcome!** Pull requests are encouraged.

## Installation

We've yet to actually publish a release. Until we do, please [build the bleeding edge code yourself.](BUILDING.md)

## Usage

The tool depends on glTF source files that are **identical** except for which materials the various
meshes reference. The proposed work flow is to export the same asset from the same digital content
creation app repeatedly, taking care to make no changes to geometry or structure between each
exported file.

Then, using the (quite primitive, as yet) command-line interface might look like:

```shell
> dist/app.js black:GizmoBlack.glb blue:GizmoBlue.glb clear:GizmoClear.glb GizmoVariational.glb
Parsing source asset: 'GizmoBlack.glb'...
Initial asset:
             Total file size: 2.4 MB
          Total texture data: 1.8 MB
  Of which is depends on tag: 0.0 kB

Parsing source asset: 'GizmoBlue.glb'...
New melded result:
             Total file size: 3.9 MB
          Total texture data: 3.3 MB
  Of which is depends on tag: 3.3 MB

Parsing source asset: 'GizmoClear.glb'...
New melded result:
             Total file size: 4.6 MB
          Total texture data: 4.0 MB
  Of which is depends on tag: 4.0 MB
Success! 4594404 bytes written to 'GizmoVariational.glb'.
```

The first source file contains 1.8 MB of textures and 0.6 MB of geometry. Subsequent source files
contribute first another 1.5 MB of textures, and then for the third variant, 1.7 MB. The geometry
of the asset remains constant.


### Asset Requirements

For assets to be meldable, they must be logically identical: contain the same meshes, with
the same mesh primitives. They may vary meaningfully only in what _materials_ are assigned
to each mesh primitive. The tool will complain if it finds disrepancies between the source
assets that are too confusing for it to work around.

During the melding process, all common data is shared, whereas varying material definitions and
textures are copied as needed. Parts of the assets that don't vary are left untouched.

Each source asset brought into the tool is identified by a _tag_, a short string, and it's
these same tags that are later used to trigger different runtime apperances.

## Building

Please see separate [BUILDING](BUILDING.md) instructions.

## Contributing

See the [CONTRIBUTING](CONTRIBUTING.md) file for how to help out.

## Credits

This tool was written by Pär Winzel and Renee Rashid with help from Susie Su and Jeremy Cytryn,
and ultimately made possible only through the hard work of others:

- The [Rust](https://www.rust-lang.org/) language & community,
- The authors of [`wasm-bindgen`](https://rustwasm.github.io/docs/wasm-bindgen/), for WebAssembly support,
- The Rust crates [`gltf`](https://github.com/gltf-rs/gltf) and
  [`serde`](https://github.com/serde-rs/serde).
- and many others...

## License

glTFVariantMeld is NIT licensed, as found in the [LICENSE](LICENSE.txt) file.

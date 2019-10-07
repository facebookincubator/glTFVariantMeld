# glTFVariantMeld

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![CircleCI](https://circleci.com/gh/facebookincubator/glTFVariantMeld/tree/master.svg?style=svg&circle-token=444333da241c0fc99a7ac8f786129f3bce774b43)](https://circleci.com/gh/facebookincubator/glTFVariantMeld/tree/master)

## Description

This tool melds multiple glTF assets, each representing a different _variant_ of a model,
into a single, compact format, implemented as a glTF extension.

A canonical example use is a retail product that's available in a range of colours, with
an application that lets a prospective customer switch between these different variants
without latency or stuttering.

We're making this internal tool publicly available with the hope of helping the glTF
ecosystem come together around a common, open format. In this prerelease version, the tool
produces files with the vendor extension
[`FB_material_variants`](https://github.com/KhronosGroup/glTF/pull/1681). We are hopeful
that the glTF community will find speedy consensus around a ratified extension.

## Installation

TODO: We've yet to actually publish an NPM package. Until we do, please [build it yourself.](BUILDING.md)

```
    > npm install -g gltf-variant-meld
```

Our aspirational roadmap includes the development of a web app which would leverage
WebAssembly to run entirely in the browser. There will also be a native CLI.

## Usage

Using the (quite primitive, as yet) command-line interface might look like:

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

This tool was written by Pär Winzell, Renee Rashid, Susie Su, and Jeremy Cytryn, and made possible
through the hard work of others:

- The [Rust](https://www.rust-lang.org/) language & community,
- The authors of [`wasm-bindgen`](https://rustwasm.github.io/docs/wasm-bindgen/), for WebAssembly support,
- The Rust crates [`gltf`](https://github.com/gltf-rs/gltf) and
  [`serde`](https://github.com/serde-rs/serde).
- and many others...

## License

glTFVariantMeld is NIT licensed, as found in the [LICENSE](LICENSE.txt) file.

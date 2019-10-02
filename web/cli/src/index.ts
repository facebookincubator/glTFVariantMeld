// Copyright (c) Facebook, Inc. and its affiliates. All Rights Reserved
//

import { Variationator, runWithVariationator } from "./wasmpkg";
import { readFileSync, writeFileSync } from "fs";
import { basename } from "path";
import { VariationalAsset } from "../node_modules/@zellski/variationator/variationator";

const GLB_MAGIC: number = 0x46546c67;

runWithVariationator(start)
  .then(() => {
    // cool
  })
  .catch(err => {
    console.error("Aborting due to error: " + err);
  });

function start(wasmpkg: Variationator) {
  let args = process.argv.slice(2);
  if (args.length < 2) {
    console.error(
      "Usage: %s [<tag>:]<input glb> [<tag>:]<input glb> <output glb>",
      basename(process.argv[1])
    );
    process.exit(1);
  }

  let inputs: [string, string][] = [];
  for (let ix = 0; ix < args.length - 1; ix++) {
    inputs.push(parseInputArg(args[ix]));
  }
  let output = args[args.length - 1];

  function parse_input(ix: number): VariationalAsset {
    let tag = inputs[ix][0];
    let file = inputs[ix][1];

    console.log("Parsing source asset: '" + file + "'...");
    let bytes = readAndValidate(file) as Uint8Array;
    return wasmpkg.VariationalAsset.wasm_from_slice(bytes, tag);
  }

  let result = parse_input(0);
  console.log("Initial asset:");
  describe_asset(result);

  for (let ix = 1; ix < inputs.length; ix++) {
    console.log();
    result = wasmpkg.VariationalAsset.wasm_meld(result, parse_input(ix));
    console.log("New melded result:");
    describe_asset(result);
  }

  let output_glb = result.wasm_glb();
  writeFileSync(output, output_glb);

  console.log("Success! %d bytes written to '%s'.", output_glb.length, output);
}

function describe_asset(asset: VariationalAsset) {
  console.log("             Total file size: " + size(asset.wasm_glb().length));
  let total = asset.wasm_metadata().total_sizes().texture_bytes;
  let variational = asset.wasm_metadata().variational_sizes().texture_bytes;
  console.log("          Total texture data: " + size(total));
  console.log("  Of which is depends on tag: " + size(variational));
}

function size(byte_count: number): string {
  if (byte_count < 1000000) {
    return (byte_count / 1000).toFixed(1) + " kB";
  }
  return (byte_count / 1000000).toFixed(1) + " MB";
}

// A file input argument can either be <tag>:<file> or plain <file>, e.g.
// matte:./models/matte.glb
// ./models/matte.gplb
function parseInputArg(arg: string): [string, string] {
  let ix = arg.indexOf(":");
  if (ix < 0) {
    return [undefined, arg];
  }
  return [arg.substring(0, ix), arg.substring(ix + 1)];
}

// Synchronously read the contents of a file, and ascertain that it's a GLB file.
function readAndValidate(file: string): Buffer {
  let contents = readFileSync(file);
  let first_word = contents.readUIntLE(0, 4);
  if (first_word === GLB_MAGIC) {
    return contents;
  }
  console.error("File %s is not a GLB file: starts with 0x%s.", file, first_word.toString(16));
  process.exit(1);
}

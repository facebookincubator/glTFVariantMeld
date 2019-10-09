## Setup

There are a number of prerequisites for building the code in this repo.

Here follow terse descriptions on how to get there.

### Setup for Native: The Rust Toolchain

- Follow [these instructions](https://www.rust-lang.org/tools/install) to install the Rust
  toolchain.

  - There are other installation paths, but `rustup` is heartily recommended.
  - Installing `rustup` should also install `cargo`, Rust's day-to-day workhorse tool.
  - Run `rustup update` both now & occasionally, to get updates to languages & components.

### Setup for Web: Node & WebAssembly

- Follow [these](https://rustwasm.github.io/wasm-pack/installer/) to install wasm-bindgen.
  - This is the tool that binds Rust and WebAssembly together.
- For sturdiness, we require specific versions of Node.js.
  - To that end, [install nvm](https://github.com/nvm-sh/nvm#install--update-script), the Node
    Version Manager.
    - Note the instructions on how to enable `nvm` for your current shell session.
  - Grab a stable node version for general use: `nvm install --lts --latest-npm`
  - Further down, you will run a script that installs a specific node version for our use.

### Fetching the Code

Grab the actual repository:

```
    > git clone https://github.com/facebookincubator/glTFVariantMeld
    > cd glTFVariantMeld
```

## Generate Rust Binaries

At this point you should be able to run `cargo build`, which will recurse into `./native` where the Rust source lives. Binaries will end up in `./target/debug/`.

## Generate WebAssembly Package

If you now try:

```
   > ./build-wasm-pkg.sh
```

you should end up with a generated NPM package in `./web/wasmpkg/`.

You can eyeball it for fun, but we don't do anything with the generated package directly. We just reference it from elsewhere, as per the next section.

## Run the Node.js test app

Now simply run:

```
    > cd web/cli
    > npm install
    > ./build-node-app.sh
```

This script should make sure you've got NVM installed, and then use it to make sure you're running the recommended Node.js version, then execute the TypeScript-blessed WebPack to generate the required files in `./dist`. Something like this:

```
    > ls -l ./dist
    -rw-r--r--  1 zell  staff    31497 Sep 17 17:05 0.app.js
    -rwxr-xr-x  1 zell  staff    31707 Sep 17 17:05 app.js
    -rw-r--r--  1 zell  staff  1010595 Sep 17 17:05 d4d094458c23e79dfea6.module.wasm
```

Finally, the script executes `dist/app.js` which actually runs the app.

Tada! You have built a native library, converted it to WebAssembly, bundled it all into executable JavaScript, and successfully run it. It's not much harder to run it in the browser, but instructions for that will come later.

// Copyright (c) Facebook, Inc. and its affiliates. All Rights Reserved
//

const path = require("path");
const webpack = require("webpack");

module.exports = {
  entry: "./src/index.ts",
  devtool: "inline-source-map",
  module: {
    rules: [
      {
        test: /\.tsx?$/,
        use: "ts-loader",
        exclude: /node_modules/
      }
    ]
  },
  plugins: [
    new webpack.BannerPlugin({
      banner: "#!/usr/bin/env node",
      entryOnly: true,
      raw: true
    })
  ],
  resolve: {
    extensions: [".tsx", ".ts", ".js"]
  },
  output: {
    filename: "app.js",
    path: path.resolve(__dirname, "dist")
  },
  mode: "development"
};

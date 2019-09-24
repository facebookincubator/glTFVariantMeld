#!/bin/bash
#
# Copyright (c) Facebook, Inc. and its affiliates. All Rights Reserved
#


NODE_VERSION="v11.15.0"
NVMSH="${HOME}/.nvm/nvm.sh"

test -f "${NVMSH}" || {
    printf "Please install NVM, the Node Version Manager. See README.md".
    exit 1
}

. "${NVMSH}"

nvm use "${NODE_VERSION}" || {
    printf "Please run: nvm install \"${NODE_VERSION}\" --latest-npm"
    printf "(If you don't have nvm install, see README.md.)"
    exit 1
}

printf "\n"
printf "\033[1;91m(Re)building with WebPack & TypeScript:\033[0m\n"
printf "\033[1;91m---------------------------------------\033[0m\n"
node node_modules/webpack-cli/bin/cli.js --target node
chmod 755 dist/app.js

printf "\n"
printf "\033[1;91mRun the application with: dist/app.js\033[0m\n"
printf "\033[1;91m---------------------------------------------------------------------------\033[0m\n"

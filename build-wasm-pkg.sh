#!/bin/bash
#
# Copyright (c) Facebook, Inc. and its affiliates. All Rights Reserved
#

SCRIPT_DIR="$(cd `dirname $BASH_SOURCE`; pwd)"

cd "${SCRIPT_DIR}/native"
wasm-pack build -d ${SCRIPT_DIR}/web/wasmpkg/

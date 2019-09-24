#!/bin/bash
#
# Copyright (c) Facebook, Inc. and its affiliates. All Rights Reserved
#

SCRIPT_DIR=`dirname $BASH_SOURCE`
cd "${SCRIPT_DIR}"

wasm-pack build -d ${SCRIPT_DIR}/web/wasmpkg/

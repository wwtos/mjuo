#!/bin/bash

OUT_DIR="../vpo-frontend/src/lib/wasm"

wasm-pack build --target web --out-dir "${OUT_DIR}" "$@"
patch "${OUT_DIR}/vpo_backend.js" vpo_backend.patch
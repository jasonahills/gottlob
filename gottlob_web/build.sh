#!/usr/bin/env bash

set -e

rm ./pkg/* || true
wasm-pack build --target web
rollup public/main.js --format iife --file ./pkg/bundle.js
cp public/* ./pkg/
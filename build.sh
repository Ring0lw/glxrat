#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

cargo build --release
cp target/release/libglxrat.so glxrat.cdylib.part
mv glxrat.cdylib.part glxrat.cdylib
echo "[+] glxrat.cdylib ready"

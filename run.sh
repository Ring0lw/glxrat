#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

exec env LD_PRELOAD="$PWD/glxrat.cdylib" glxgears "$@"

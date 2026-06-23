#!/usr/bin/env bash
set -euo pipefail

elf_path="$1"
shift
gba_path="${elf_path}.gba"

agb-gbafix "$elf_path" -o "$gba_path"
exec mgba-qt "$gba_path" "$@"

#!/bin/sh
# Extract one verbatim block from PLAN.md into a file.
# Usage: sh tools/extract.sh <block-id> <destination-path>
# A block is the fenced code between the lines
#   <!-- BEGIN BLOCK id -->  and  <!-- END BLOCK id -->
set -eu
if [ "$#" -ne 2 ]; then
  echo "usage: sh tools/extract.sh <block-id> <destination-path>" >&2
  exit 2
fi
id="$1"
dest="$2"
plan="$(dirname "$0")/../PLAN.md"
mkdir -p "$(dirname "$dest")"
tmp="$dest.extract.$$"
if awk -v begin="<!-- BEGIN BLOCK $id -->" -v end="<!-- END BLOCK $id -->" '
  $0 == begin { inside = 1; fence = 0; found = 1; next }
  $0 == end { inside = 0; next }
  inside && fence == 0 && /^```/ { fence = 1; next }
  inside && fence == 1 && /^```$/ { fence = 2; next }
  inside && fence == 1 { print }
  END { if (!found) exit 3 }
' "$plan" > "$tmp"; then
  mv "$tmp" "$dest"
else
  rm -f "$tmp"
  echo "block not found in PLAN.md: $id" >&2
  exit 3
fi

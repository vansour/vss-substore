#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FONT_SOURCE="${1:-https://hyperos.mi.com/font-download/MiSans.zip}"
OUTPUT_PATH="${2:-$ROOT_DIR/frontend/assets/fonts/submora-misans-ui-vf.woff2}"

for dependency in curl unzip pyftsubset python3; do
  if ! command -v "$dependency" >/dev/null 2>&1; then
    echo "missing dependency: $dependency" >&2
    exit 1
  fi
done

WORK_DIR="$(mktemp -d)"
cleanup() {
  rm -rf "$WORK_DIR"
}
trap cleanup EXIT

ZIP_PATH="$WORK_DIR/MiSans.zip"
EXTRACT_DIR="$WORK_DIR/extracted"
SUBSET_TEXT="$WORK_DIR/subset-text.txt"

if [[ "$FONT_SOURCE" =~ ^https?:// ]]; then
  curl -L "$FONT_SOURCE" -o "$ZIP_PATH"
else
  cp "$FONT_SOURCE" "$ZIP_PATH"
fi

unzip -q "$ZIP_PATH" -d "$EXTRACT_DIR"

SOURCE_FONT="$(find "$EXTRACT_DIR" -type f -name 'MiSansVF.ttf' -print -quit)"
if [[ -z "$SOURCE_FONT" ]]; then
  echo "MiSansVF.ttf not found in extracted archive" >&2
  exit 1
fi

python3 - "$ROOT_DIR/frontend/src" "$SUBSET_TEXT" <<'PY'
from pathlib import Path
import sys

source_root = Path(sys.argv[1])
output_path = Path(sys.argv[2])

ascii_chars = "".join(chr(i) for i in range(0x20, 0x7F)) + "\u00A0\u00B7\u2026"
text = ascii_chars

for path in sorted(source_root.rglob("*.rs")):
    text += path.read_text(encoding="utf-8")

output_path.write_text("".join(dict.fromkeys(text)), encoding="utf-8")
PY

mkdir -p "$(dirname "$OUTPUT_PATH")"

pyftsubset "$SOURCE_FONT" \
  --output-file="$OUTPUT_PATH" \
  --flavor=woff2 \
  --text-file="$SUBSET_TEXT" \
  --layout-features='*' \
  --passthrough-tables \
  --name-IDs='*' \
  --name-legacy \
  --symbol-cmap \
  --notdef-glyph \
  --notdef-outline \
  --recommended-glyphs \
  --drop-tables=''

echo "wrote $(du -h "$OUTPUT_PATH" | cut -f1) to $OUTPUT_PATH"

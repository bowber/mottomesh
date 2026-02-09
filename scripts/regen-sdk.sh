#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCHEMA_DIR="$ROOT_DIR/schema"
TS_SDK_DIR="$SCHEMA_DIR/generated/typescript"

BUMP_LEVEL="${1:-patch}"

echo "==> Regenerating Motto SDKs (bump: ${BUMP_LEVEL})"
cd "$SCHEMA_DIR"
motto generate --targets rust,typescript --output generated --force
motto lock --bump "$BUMP_LEVEL"

echo "==> Building generated TypeScript SDK"
cd "$TS_SDK_DIR"
npm install
npm run build

echo "==> Done"

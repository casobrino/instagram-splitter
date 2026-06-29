#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_NAME="Instagram Photo Splitter"
BUNDLE_ID="com.instagram-splitter.app"
BINARY_NAME="instagram-splitter"
DIST_DIR="$ROOT/dist"
APP_DIR="$DIST_DIR/$APP_NAME.app"

cd "$ROOT"

echo "Compilando en modo release..."
cargo build --release

echo "Creando bundle .app..."
rm -rf "$APP_DIR"
mkdir -p "$APP_DIR/Contents/MacOS"
mkdir -p "$APP_DIR/Contents/Resources"

cp "$ROOT/target/release/$BINARY_NAME" "$APP_DIR/Contents/MacOS/$BINARY_NAME"
chmod +x "$APP_DIR/Contents/MacOS/$BINARY_NAME"
cp "$ROOT/packaging/macos/Info.plist" "$APP_DIR/Contents/Info.plist"

# Evita el aviso de "desarrollador no identificado al copiar el binario.
xattr -cr "$APP_DIR" 2>/dev/null || true

echo ""
echo "Listo: $APP_DIR"
echo ""
echo "Para instalar:"
echo "  cp -R \"$APP_DIR\" /Applications/"
echo ""
echo "O arrastra la app desde Finder:"
echo "  open \"$DIST_DIR\""

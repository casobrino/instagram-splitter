# Instagram Photo Splitter

Desktop app to prepare photos for Instagram. Split a horizontal image into
several posts, build a Polaroid-style framed photo, or create a cinematic
multi-strip collage — all at Instagram's maximum feed sizes.

Written in Rust with [egui](https://github.com/emilk/egui).

> 🇪🇸 **La versión en español está más abajo** — [ir a español](#español).

---

## Download

Grab a prebuilt binary from the [Releases page](../../releases):

| Platform | File |
|----------|------|
| Windows (x64) | `instagram-splitter-windows-x86_64.zip` |
| macOS (Apple Silicon) | `instagram-splitter-macos-aarch64.zip` |
| macOS (Intel) | `instagram-splitter-macos-x86_64.zip` |

- **Windows:** unzip and run `instagram-splitter.exe`.
- **macOS:** unzip and move `Instagram Photo Splitter.app` to `/Applications`.
  The first launch may require right-click → **Open** (unsigned app).

## Features

### Splitter tab

Splits a **horizontal** photo into 2 or 3 ready-to-post images:

| Mode | Result |
|------|--------|
| 2 photos 1:1 | Two squares 1080×1080 |
| 2 photos 4:5 | Two portraits 1080×1350 |
| 3 photos 4:5 | Three portraits 1080×1350 |

- Drag the box to choose the crop area
- Adjust the box width with the slider
- Export all photos as JPG into a folder

### Polaroid tab

Creates a Polaroid-style framed image (thin border on top/sides, thick border
at the bottom). Accepts photos in **any orientation**.

**Output formats:** 1:1 (1080×1080), 4:5 (1080×1350), 9:16 (1080×1920),
1.91:1 (1080×566).

- Frame background color (white, cream, gray, black or custom)
- Bottom border thickness
- Zoom, rotation (−45° to +45°) and photo position
- Drag the image in the preview to move it
- Ctrl + mouse wheel (⌘ + wheel on Mac) to zoom in the preview

### Cine tab

Builds a **cinematic collage** made of horizontal strips, each with its own
image, for a film-still look.

| Canvas | Strips | Per-strip size | Aspect |
|--------|--------|----------------|--------|
| 4:5 (1080×1350) | 3 | 1080×450 | 2.4:1 (cinema) |
| 4:5 (1080×1350) | 4 | 1080×337 | 3.2:1 (ultra-wide) |
| 9:16 (1080×1920) | 3 | 1080×640 | ≈16:9 |
| 9:16 (1080×1920) | 4 | 1080×480 | 2.25:1 |

- Pick the output canvas (4:5 or 9:16) and the number of strips (3 or 4)
- Each strip starts as an empty slot — **click a strip** (or **Add image**) to
  load a photo into it
- Click an already-filled strip to **replace** its image
- **Drag** inside a strip to reposition it; **Ctrl + wheel** to zoom that strip
- **Generate** once every strip has an image, then **Save** to export
  `cine_1.jpg`, `cine_2.jpg`, …

## Build from source

Requires [Rust](https://rustup.rs/) (2021 edition). On macOS you also need the
Xcode command line tools to package the `.app`.

```bash
cargo run --release
```

### Package a macOS .app locally

```bash
./scripts/package-macos.sh
```

Produces `dist/Instagram Photo Splitter.app`. To install:

```bash
cp -R "dist/Instagram Photo Splitter.app" /Applications/
```

## Releasing (maintainers)

Pushing a version tag triggers the [release workflow](.github/workflows/release.yml),
which builds Windows + macOS binaries and attaches them to a GitHub Release:

```bash
git tag v1.0.0
git push origin v1.0.0
```

You can also run the workflow manually from the **Actions** tab.

## Project structure

```
src/
  main.rs      # Main app and tabs
  splitter.rs  # Horizontal photo splitting
  polaroid.rs  # Polaroid frame and export
  cine.rs      # Cinematic multi-strip collage
scripts/
  package-macos.sh
packaging/
  macos/Info.plist
.github/
  workflows/release.yml
```

## License

Personal project. Add a license if you publish it.

---

<a name="español"></a>

# Instagram Photo Splitter (Español)

Aplicación de escritorio para preparar fotos para Instagram. Divide una imagen
horizontal en varias publicaciones, crea una foto con marco estilo Polaroid, o
arma un collage cinematográfico de varias tiras — todo en los tamaños máximos
del feed.

Escrita en Rust con [egui](https://github.com/emilk/egui).

## Descargar

Descarga un binario ya compilado desde la [página de Releases](../../releases):

| Plataforma | Archivo |
|------------|---------|
| Windows (x64) | `instagram-splitter-windows-x86_64.zip` |
| macOS (Apple Silicon) | `instagram-splitter-macos-aarch64.zip` |
| macOS (Intel) | `instagram-splitter-macos-x86_64.zip` |

- **Windows:** descomprime y ejecuta `instagram-splitter.exe`.
- **macOS:** descomprime y mueve `Instagram Photo Splitter.app` a
  `/Aplicaciones`. La primera vez puede que tengas que hacer clic derecho →
  **Abrir** (app sin firmar).

## Funciones

### Tab Recortes

Divide una foto **horizontal** en 2 o 3 imágenes listas para publicar:

| Modo | Resultado |
|------|-----------|
| 2 fotos 1:1 | Dos cuadrados 1080×1080 |
| 2 fotos 4:5 | Dos retratos 1080×1350 |
| 3 fotos 4:5 | Tres retratos 1080×1350 |

- Arrastra el recuadro para elegir la zona a recortar
- Ajusta el ancho del recuadro con el slider
- Exporta todas las fotos como JPG en una carpeta

### Tab Polaroid

Crea una imagen con marco tipo Polaroid (borde fino arriba y a los lados, borde
grueso abajo). Acepta fotos en **cualquier orientación**.

**Formatos de salida:** 1:1 (1080×1080), 4:5 (1080×1350), 9:16 (1080×1920),
1.91:1 (1080×566).

- Color de fondo del marco (blanco, crema, gris, negro o personalizado)
- Grosor del borde inferior
- Zoom, rotación (−45° a +45°) y posición de la foto
- Arrastra la imagen en la vista previa para moverla
- Ctrl + rueda del ratón (⌘ + rueda en Mac) para zoom en la vista previa

### Tab Cine

Arma un **collage cinematográfico** formado por tiras horizontales, cada una con
su propia imagen, para un aspecto de fotograma de película.

| Lienzo | Tiras | Tamaño por tira | Aspecto |
|--------|-------|-----------------|---------|
| 4:5 (1080×1350) | 3 | 1080×450 | 2.4:1 (cine) |
| 4:5 (1080×1350) | 4 | 1080×337 | 3.2:1 (ultra-panorámico) |
| 9:16 (1080×1920) | 3 | 1080×640 | ≈16:9 |
| 9:16 (1080×1920) | 4 | 1080×480 | 2.25:1 |

- Elige el lienzo de salida (4:5 o 9:16) y el número de tiras (3 o 4)
- Cada tira empieza como un espacio vacío — **haz clic en una tira** (o en
  **Añadir imagen**) para cargar una foto en ella
- Haz clic en una tira ya rellena para **cambiar** su imagen
- **Arrastra** dentro de una tira para reposicionarla; **Ctrl + rueda** para
  hacer zoom en esa tira
- **Generar** cuando todas las tiras tengan imagen, y luego **Guardar** para
  exportar `cine_1.jpg`, `cine_2.jpg`, …

## Compilar desde el código

Requiere [Rust](https://rustup.rs/) (edición 2021). En macOS también necesitas
las herramientas de línea de comandos de Xcode para empaquetar la `.app`.

```bash
cargo run --release
```

### Empaquetar la .app de macOS localmente

```bash
./scripts/package-macos.sh
```

Genera `dist/Instagram Photo Splitter.app`. Para instalarla:

```bash
cp -R "dist/Instagram Photo Splitter.app" /Applications/
```

## Publicar versiones (mantenedores)

Al subir un tag de versión se ejecuta el [workflow de release](.github/workflows/release.yml),
que compila los binarios de Windows + macOS y los adjunta a un Release de GitHub:

```bash
git tag v1.0.0
git push origin v1.0.0
```

También puedes ejecutar el workflow manualmente desde la pestaña **Actions**.

## Estructura del proyecto

```
src/
  main.rs      # App principal y tabs
  splitter.rs  # División de fotos horizontales
  polaroid.rs  # Marco Polaroid y exportación
  cine.rs      # Collage cinematográfico de varias tiras
scripts/
  package-macos.sh
packaging/
  macos/Info.plist
.github/
  workflows/release.yml
```

## Licencia

Proyecto personal. Añade una licencia si lo publicas.

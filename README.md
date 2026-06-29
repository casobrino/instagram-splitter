# Instagram Photo Splitter

Aplicación de escritorio para preparar fotos para Instagram. Convierte una imagen horizontal en varias publicaciones, o crea una foto con marco estilo Polaroid en los tamaños máximos del feed.

Escrita en Rust con [egui](https://github.com/emilk/egui).

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

Crea una imagen con marco tipo Polaroid (borde fino arriba y a los lados, borde grueso abajo). Acepta fotos en **cualquier orientación**.

**Formatos de salida:**

| Formato | Dimensiones |
|---------|-------------|
| 1:1 | 1080×1080 |
| 4:5 | 1080×1350 |
| 9:16 | 1080×1920 |
| 1.91:1 | 1080×566 |

**Ajustes disponibles:**

- Color de fondo del marco (blanco, crema, gris, negro o personalizado)
- Grosor del borde inferior
- Zoom, rotación (−45° a +45°) y posición de la foto
- Arrastra la imagen en la vista previa para moverla
- Ctrl + rueda del ratón (⌘ + rueda en Mac) para zoom en la vista previa

## Requisitos

- [Rust](https://rustup.rs/) (edición 2021)

En macOS, para empaquetar la app también necesitas las herramientas de línea de comandos de Xcode.

## Compilar y ejecutar

```bash
cargo run --release
```

## Empaquetar en macOS (.app)

```bash
./scripts/package-macos.sh
```

Genera `dist/Instagram Photo Splitter.app`. Para instalarla:

```bash
cp -R "dist/Instagram Photo Splitter.app" /Applications/
```

## Uso rápido

### Recortes

1. Abre el tab **Recortes**
2. Pulsa **Abrir imagen** (debe ser una foto horizontal)
3. Elige el modo (2×1:1, 2×4:5 o 3×4:5)
4. Ajusta el recuadro arrastrando o con el slider
5. Pulsa **Convertir** y luego **Guardar imágenes**

### Polaroid

1. Abre el tab **Polaroid**
2. Pulsa **Abrir imagen**
3. Elige formato, color de marco y ajusta la foto
4. Pulsa **Generar** y luego **Guardar**

## Estructura del proyecto

```
src/
  main.rs      # App principal y tabs
  splitter.rs  # División de fotos horizontales
  polaroid.rs  # Marco Polaroid y exportación
scripts/
  package-macos.sh
packaging/
  macos/Info.plist
```

## Licencia

Proyecto personal. Añade una licencia si lo publicas.

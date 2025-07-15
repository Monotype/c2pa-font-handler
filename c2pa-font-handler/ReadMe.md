# Font I/O

- [Features](#features)
- [Examples](#examples)
  - [`render_thumbnails`](#render_thumbnails)
  - [`stub_dsig`](#stub_dsig)
  - [`woff1`](#woff1)

This crate was brought in and adapted from the implementation found in [font_io.rs](https://github.com/Monotype/c2pa-rs/blob/monotype/fontSupport/sdk/src/asset_handlers/font_io.rs) and [sfnt_io.rs](https://github.com/Monotype/c2pa-rs/blob/monotype/fontSupport/sdk/src/asset_handlers/sfnt_io.rs) in the Monotype fork of the `c2pa-rs` repository.

The use of "font I/O" is a bit strong, as it doesn't really do much at the moment. It has the ability read in a font and write it back out ensuring no new tables were added/removed.

For more information, check out [lib.rs](./src/lib.rs).

## Features

There are a few features available for this crate:

Feature|Note|On by Default
-|-|-
`compression`|Turns on support for compression, using the `flate` feature|❌ No
`flate`|Compiles the `flate2` crate|❌ No
`png-thumbnails`|Adds the ability to create PNG thumbnails for SFNT (and WOFF1) files|❌ No
`svg-thumbnails`|Adds the ability to create SVG thumbnails for SFNT (and WOFF1) files|✅ Yes
`thumbnails`|Use of `cosmic-text` crate for generating thumbnails; `png-thumbnails` and/or `svg-thumbnails` turn this on when used.|✅ Yes
`woff`|Turns on support for WOFF fonts (this is currently a work in progress)|❌ No

## Examples

### `render_thumbnails`

The [render_thumbnails](./examples/render_thumbnail.rs) example is provided to show how to generate either an SVG or a PNG thumbnail for a font (currently only SFNT files supported). To run the example:

```shell
cargo run --features="svg-thumbnails png-thumbnails" --example "render_thumbnail" -- -t svg --input "path/to/font.otf" --output "path/to/thumbnail.svg"
```

> NOTE: Since `woff` feature is not on by default, you must also specify `woff` to generate a thumbnail for a WOFF1 file.

### `stub_dsig`

The [stub_dsig](./examples/stub_dsig.rs) example is provided to show how to read a font, create a stub DSIG table in place of a pre-existing one. To run the example:

```shell
cargo run --example "stub_dsig" -- --input "path/to/font.otf" --output "path/to/new/font.otf"
```

### `woff1`

The [woff1](./examples/woff1.rs) example is provided to show how to read a WOFF1 file. To run the example:

```shell
cargo run --features="woff" --example "woff1" -- --input "path/to/font.otf"
```

# Font I/O

- [Features](#features)
- [Examples](#examples)

This crate was brought in and adapted from the implementation found in [font_io.rs](https://github.com/Monotype/c2pa-rs/blob/monotype/fontSupport/sdk/src/asset_handlers/font_io.rs) and [sfnt_io.rs](https://github.com/Monotype/c2pa-rs/blob/monotype/fontSupport/sdk/src/asset_handlers/sfnt_io.rs) in the Monotype fork of the `c2pa-rs` repository.

The use of "font I/O" is a bit strong, as it doesn't really do much at the moment. It has the ability read in a font and write it back out ensuring no new tables were added/removed. And with the use of the `FontDSIGStubber` trait, the font can be updated to have a stub or fake DSIG table.

For more information, check out [lib.rs](./src/lib.rs).

## Features

There are a few features available for this crate:

Feature|Note|On by Default
-|-|-
`flate`|Compiles the `flate2` crate|No
`compression`|Turns on support for compression, using the `flate` feature|No
`woff`|Turns on support for WOFF fonts (this is currently a work in progress)|No

## Examples

The [stub_dsig](./examples/stub_dsig.rs) example is provided to show how to read a font, create a stub DSIG table in place of a pre-existing one. To try it out:

```shell
cargo run --example stub_dsig -- --input path/to/font.ttf --output path/to/new/font.ttf
```

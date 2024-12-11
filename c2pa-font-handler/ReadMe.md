# Font I/O

- [Examples](#examples)

This crate was brought in and adapted from the implementation found in [font_io.rs](https://github.com/Monotype/c2pa-rs/blob/monotype/fontSupport/sdk/src/asset_handlers/font_io.rs) and [sfnt_io.rs](https://github.com/Monotype/c2pa-rs/blob/monotype/fontSupport/sdk/src/asset_handlers/sfnt_io.rs) in the Monotype fork of the `c2pa-rs` repository.

The use of "font I/O" is a bit strong, as it doesn't really do much at the moment. It has the ability read in a font and write it back out ensuring no new tables were added/removed. And with the use of the `FontDSIGStubber` trait, the font can be updated to have a stub or fake DSIG table.

For more information, check out [lib.rs](./src/lib.rs).

## Examples

The [stub_dsig](./examples/stub_dsig.rs) example is provided to show how to read a font, create a stub DSIG table in place of a pre-existing one. To try it out:

```shell
cargo run --example stub_dsig -- --input path/to/font.ttf --output path/to/new/font.ttf
```

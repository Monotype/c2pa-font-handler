# Changelog

All changes to this project are documented in this file.

This project adheres to [Semantic Versioning](https://semver.org), except that – as is typical in the Rust community – the minimum supported Rust version may be increased without a major version increase.

Do not manually edit this file. It will be automatically updated when a new release is published.

## 0.5.0

16 July 2025

* C2PA-718/C2P-143: Ensure the C2PA table is written at the end of the stream. ([#60](https://github.com/Monotype/c2pa-font-handler/pull/60))
* C2PA-689/C2PA-690: (MINOR) Generate thumbnails for WOFF1 files. ([#61](https://github.com/Monotype/c2pa-font-handler/pull/61))

## 0.4.5

08 July 2025

* Workflow failure - dev - 9611d490758fde603e7c96ae78f9a30777c36e12 ([#56](https://github.com/Monotype/c2pa-font-handler/pull/56))

## 0.4.4

01 July 2025

* Workflow failure - dev - f066403f7e125774f84a31bbdff407c6fdb09268 ([#52](https://github.com/Monotype/c2pa-font-handler/pull/52))
* C2PA-708: Check for stubbed DSIG ([#51](https://github.com/Monotype/c2pa-font-handler/pull/51))
* C2PA-688: Ability to convert a WOFF to SFNT ([#50](https://github.com/Monotype/c2pa-font-handler/pull/50))
* C2PA-695: Correctly update WOFF header for length, numTables, and totalSfntSize ([#49](https://github.com/Monotype/c2pa-font-handler/pull/49))
* C2PA-687: Ability to render thumbnails for SFNT files

## 0.4.3

23 May 2025

* C2PA-674: Ability to update C2PA record in a WOFF1 font ([#43](https://github.com/Monotype/c2pa-font-handler/pull/43))
* C2PA-678: Add benchmarks for SFNT and WOFF1. ([#42](https://github.com/Monotype/c2pa-font-handler/pull/42))
* C2PA-275: Only take a portion of the stream and read exact amount. ([#41](https://github.com/Monotype/c2pa-font-handler/pull/41))
* C2PA-275: Compression for WOFF tables (mainly C2PA) ([#40](https://github.com/Monotype/c2pa-font-handler/pull/40))

## 0.4.2

25 April 2025

* Workflow failure - dev - 9a2b7a386e96eae8180f65322fef106e0b60c066 ([#35](https://github.com/Monotype/c2pa-font-handler/pull/35))

## 0.4.1

09 April 2025

* Workflow failure - dev - d08b9c153f71ae291e5ac0a0f0319bfff1b32139 ([#30](https://github.com/Monotype/c2pa-font-handler/pull/30))
* C2PA-275: Compression support for woff ([#29](https://github.com/Monotype/c2pa-font-handler/pull/29))

## 0.4.0

07 March 2025

* C2PA-629: (MINOR) Adds the ability to read chunks and get their positions for fonts ([#25](https://github.com/Monotype/c2pa-font-handler/pull/25))

## 0.3.1

04 March 2025

* Explicitly install the rustup toolchain, to work with v1.28
* CI: Workflow failure - .github/workflows/nightly.yml - 1ccb489b28d0e885d7c519318ad3d30811193f10

## 0.3.0

28 February 2025

* C2PA-628: (MINOR) Fix the checksum calculation of the SFNT C2PA table ([#18](https://github.com/Monotype/c2pa-font-handler/pull/18))
* C2PA-623: Initial support for reading/writing WOFF for C2PA ([#14](https://github.com/Monotype/c2pa-font-handler/pull/14))

## 0.2.1

27 February 2025

* Don't use stable for the entire action, only during install of tool
* Revert the Cargo.lock to version 3.
* CI: Workflow failure - .github/workflows/nightly.yml - 50e2c080ac716ed1e348f83b2be5d6631198704f

## 0.2.0

24 February 2025

* C2PA-626: Override toolchain channel in the directory for the cargo-edit install during GH workflow ([#9](https://github.com/Monotype/c2pa-font-handler/pull/9))
* C2PA-626: Bring in GH workflows/actions into the repo. ([#7](https://github.com/Monotype/c2pa-font-handler/pull/7))
* C2PA-615: (MINOR) Expose access to tables ([#5](https://github.com/Monotype/c2pa-font-handler/pull/5))
* PI-203: Mention the use of `MAJOR` and `MINOR`. ([#4](https://github.com/Monotype/c2pa-font-handler/pull/4)) (none)


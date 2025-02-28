# Changelog

All changes to this project are documented in this file.

This project adheres to [Semantic Versioning](https://semver.org), except that – as is typical in the Rust community – the minimum supported Rust version may be increased without a major version increase.

Do not manually edit this file. It will be automatically updated when a new release is published.

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


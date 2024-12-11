# Contributing

- [Issues](#issues)
- [Pull Requests](#pull-requests)
  - [PR Naming](#pr-naming)
  - [PR Labels](#pr-labels)
  - [PR Descriptions](#pr-descriptions)
- [Branching](#branching)
  - [Development branches](#development-branches)
  - [dev](#dev)
  - [main](#main)
- [Documentation](#documentation)
- [Unit Test Coverage](#unit-test-coverage)
- [Merging](#merging)
- [Git Hooks](#git-hooks)

We welcome contributions to this project.

We ask that you understand the following guidelines before you begin.

All communications regarding this repository are governed by the
[Monotype Contributor Covenant Code of Conduct](https://github.com/Monotype/.github/blob/main/CODE_OF_CONDUCT.md).

## Issues

Bugs, feature requests and questions should be discussed using GitHub Issues.

## Pull Requests

All pull reviews require a single reviewer, preferably picked at random from the
development team tasked with working on it.

Github actions should verify the build and unit test coverage.  A minimum 90%
code coverage is required.

Constructive feedback is always expected, with comments regarding personal taste
prefixed with `nit:` to help differentiate necessary changes from ones of preference.

### PR Naming

The title of all PRs should follow the `<issue-id>: <description>` pattern, for
instance:

- `789: A useful description of the feature, bugfix, and/or enhancement`

### PR Labels

All PRs should have at least *one* label, with the primary labels being:

- `enhancement` for stories/spikes/tech-debt
- `bug` for bugs
- `documentation` for PRs which only include documentation changes

### PR Descriptions

The PR templates includes a number of sections:

- *Issues*
  - A link to the Issue(s) which this PR addresses should be included here
- *Checklist*
  - This includes a number of things which may need to be addressed for each PR
  - If an item is accomplished (for instance, the PR was labeled appropriately)
    modify the checked box for that item from `[ ]` to `[X]` to show that it has
    been handled
- *Notes for Reviewers*
  - This section should give a high level description of the PR and how best to
    inspect the changes
- *Verification Instructions*
  - This section should give step-by-step instructions for verifying that the
    code changes are working

## Branching

Branches in this repository follow the [gitflow branching workflow](https://www.atlassian.com/git/tutorials/comparing-workflows/gitflow-workflow).

### Development branches

All development branches should follow the `<type>/<issue-id>/<description>` naming pattern; for example:

- Stories: `feature/123/myStoryDescription`
- Bugs: `fix/234/myBugDescription`
- Spikes: `spike/345/mySpikeDescription`
- Tech Debt: `td/456/myTechDebtDescription`

In all cases, the final `<branchDescription>` piece should be written in camelCase.

### dev

The `dev` branch is the base branch from which all new feature branches are created.

It is the default branch for feature and fix PRs to merge into, and always contains
the most up-to-date code.

Feature and fix branches should have the cleanest possible commit history.

### main

The `main` branch contains stable, fully-tested code.

## Documentation

All code should be fully documented with in-code comments explaining low-level
functionality and directory-level `ReadMe.md` documents explaining high-level
architecture.

## Unit Test Coverage

All code which can meaningfully be covered by unit tests should be.

Unit test files should follow the `<Module name>_test.rs` naming scheme, where
`<Module name>` is the file name of the module (e.g., `utils.rs`) the unit tests
test. The unit tests should reside as a sibling to the module. And the module
should include with:

```rust
#[cfg(test)]
#[path = "module_test.rs"]
mod tests;
```

## Merging

Once the PR has been reviewed, approved & passes all automated checks, it will
be merged by the maintainers according to the following rubric:

Source Branch|Target Branch|Merge Operation|Notes
-|-|-|-
*any*|`main`|Merge|Preserves development history of release in the `main` branch
hotfix branches: `hotfix/*` |*any*|Merge|Permits the same Hotfix to merge to multiple target branches
development branches: `fix/*`, `feature/*`, `td/*`, `spike/*`|`dev`|Squash and merge|Before confirming, verify that the commit title is the same as the PR title, and the commit description is useful.

## Git Hooks

The [.rusty-hook.toml](./.rusty-hook.toml) file is intended to be used with the [rusty-hook](https://crates.io/crates/rusty-hook) tool.
To setup the hooks run:

```shell
cargo install rusty-hook
rusty-hook init
```

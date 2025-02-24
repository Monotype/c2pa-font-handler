#!/usr/bin/env bash

#  Copyright 2025 Monotype Imaging Inc.
#  
#  Licensed under the Apache License, Version 2.0 (the "License");
#  you may not use this file except in compliance with the License.
#  You may obtain a copy of the License at
#  
#     http://www.apache.org/licenses/LICENSE-2.0
#  
#  Unless required by applicable law or agreed to in writing, software
#  distributed under the License is distributed on an "AS IS" BASIS,
#  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
#  See the License for the specific language governing permissions and
#  limitations under the License.

# Stores the specified version into the version header file.
# Expects:
#   $WORKSPACE_VERSION - workspace version
#   $PREVIOUS_VERSION_TAG - previous version tag
#   $BASE_REPO_URI - base URI of the repository this belongs to

# Validate required environment vars
if [[ ! -v WORKSPACE_VERSION ]]; then
  echo "::error::WORKSPACE_VERSION not specified"
  exit 1
elif [[ ! -v PREVIOUS_VERSION_TAG ]]; then
  echo "::error::PREVIOUS_VERSION_TAG not specified"
  exit 1
elif [[ ! -v BASE_REPO_URI ]]; then
  echo "::error::BASE_REPO_URI not specified"
  exit 1
fi

# Validate workspace version is a semver
semverRegex="^(0|[1-9][0-9]*)\\.(0|[1-9][0-9]*)\\.(0|[1-9][0-9]*)(\\-[0-9A-Za-z-]+(\\.[0-9A-Za-z-]+)*)?(\\+[0-9A-Za-z-]+(\\.[0-9A-Za-z-]+)*)?$"
if [[ ! $WORKSPACE_VERSION =~ $semverRegex ]]; then
  echo "::error::WORKSPACE_VERSION is not a semver version string"
  exit 1
fi

# Grab all past commits since the last version tag
GITHUB_OUTPUT=$(git log --format="* %s" $PREVIOUS_VERSION_TAG..HEAD --no-merges | { grep -v "(IGNORE)" || :; })

# If no changes, we're done.
if [ -z "$GITHUB_OUTPUT" ]; then
  echo "::debug::No changes since last version tag, exiting."
  exit 0
fi

# Create a CHANGELOG.md file if one doesn't yet exist
if ! [ -f CHANGELOG.md ]; then
  echo "# Changelog" > CHANGELOG.md
  echo "" >> CHANGELOG.md
  echo "All changes to this project are documented in this file." >> CHANGELOG.md
  echo "" >> CHANGELOG.md
  echo "This project adheres to [Semantic Versioning](https://semver.org), except that – as is typical in the Rust community – the minimum supported Rust version may be increased without a major version increase." >> CHANGELOG.md
  echo "" >> CHANGELOG.md
  echo "Do not manually edit this file. It will be automatically updated when a new release is published." >> CHANGELOG.md
  echo "" >> CHANGELOG.md
fi

# Add the new entries into the changelog.
SED_EXPRESSION="s_\(#([0-9]+)\)_([#\1]($BASE_REPO_URI/pull/\1)\)_"
(head -8 CHANGELOG.md && echo "## $WORKSPACE_VERSION" && echo "" && date "+%d %B %Y" && echo "" && (echo "$GITHUB_OUTPUT" | sed -E $SED_EXPRESSION) && echo "" && tail -n +9 CHANGELOG.md) > CHANGELOG.new.md
mv CHANGELOG.new.md CHANGELOG.md

echo "::debug::Changelog updated for ${WORKSPACE_VERSION}"

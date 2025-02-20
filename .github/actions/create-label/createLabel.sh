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
#   $ LABEL - name of the label to create
#   $ COLOR - color of the label (e.g. "#ff0000")
#   $ DESCRIPTION - description of the label
#   $ FORCE - whether to overwrite the label if it already exists

# Validate required environment vars
if [[ ! -v LABEL ]]; then
    echo "::error::LABEL not specified"
    exit 1
elif [[ ! -v COLOR ]]; then
    echo "::error::COLOR not specified"
    exit 1
elif [[ ! -v DESCRIPTION ]]; then
    echo "::error::DESCRIPTION not specified"
    exit 1
fi

FORCE=${FORCE:-false}

REPO=$(gh repo view --json nameWithOwner --jq '.nameWithOwner')
LABEL_EXISTS=$(gh api "repos/$REPO/labels/$LABEL" --silent || echo "false")

if [ "$LABEL_EXISTS" != "false" ] && [ "$FORCE" != "true" ]; then
    echo "::warning::Label $LABEL already exists. Use FORCE=true to overwrite."
    exit 0
elif [ "$LABEL_EXISTS" != "false" ] && [ "$FORCE" == "true" ]; then
    gh api "repos/$REPO/labels/$LABEL" -X DELETE
    gh api "repos/$REPO/labels" -f name="$LABEL" -f color="$COLOR" -f description="$DESCRIPTION"
    echo "::debug::Label $LABEL updated"
else
    gh label create "$LABEL" -c "$COLOR" -d "$DESCRIPTION"
    echo "::debug::Label $LABEL created"
    LABEL_INFO=$(gh api "repos/$REPO/labels/$LABEL")
    echo "::set-output name=label_id::$(echo $LABEL_INFO | jq -r '.id')"
fi

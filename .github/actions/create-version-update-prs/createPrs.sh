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
#   $VERSION - semver version, prefixed by v
#   $WORKFLOW - name of workflow which caused this to be called
#   $TRIGGER_SHA - sha commit which triggered the workflow
#   $PRIMARY_BRANCH - name of primary branch to PR to
#   $DEVELOPMENT_BRANCH - name of development branch to PR to
#   $ASSIGNEE - github id of person to assign PRs to

# Validate required environment vars
if [[ ! -v VERSION ]]; then
  echo "::error::VERSION not specified"
  exit 1
elif [[ ! -v WORKFLOW ]]; then
  echo "::error::WORKFLOW not specified"
  exit 1
elif [[ ! -v TRIGGER_SHA ]]; then
  echo "::error::TRIGGER_SHA not specified"
  exit 1
elif [[ ! -v PRIMARY_BRANCH ]]; then
  echo "::error::PRIMARY_BRANCH not specified"
  exit 1
elif [[ ! -v DEVELOPMENT_BRANCH ]]; then
  echo "::error::DEVELOPMENT_BRANCH not specified"
  exit 1
elif [[ ! -v ASSIGNEE ]]; then
  echo "::error::ASSIGNEE not specified"
  exit 1
fi

# Create a markdown file which will act as the body.
bodyFile=prBodyFormat.md
echo "# Workspace Version Update" >> $bodyFile
echo "Updating workspace version to \`${VERSION}\`." >> $bodyFile
echo "## Details" >> $bodyFile
echo "- Created by workflow: *${WORKFLOW}*" >> $bodyFile
echo "- Triggered by \`${TRIGGER_SHA}\` commit" >> $bodyFile
echo "> **note:** Version update PRs (like this one) should always be _merged_ with a commit (not with a squash)." >> $bodyFile

# Create two PRs, one to the primary branch, and one to the development branch.
gh pr create -B "${PRIMARY_BRANCH}" -a "${ASSIGNEE}" -t "Workspace Version Update - ${PRIMARY_BRANCH} - ${VERSION}" -l "enhancement" -F $bodyFile
echo "::debug::Version PR created to ${PRIMARY_BRANCH}"
gh pr create -B "${DEVELOPMENT_BRANCH}" -a "${ASSIGNEE}" -t "Workspace Version Update - ${DEVELOPMENT_BRANCH} - ${VERSION}" -l "enhancement" -F $bodyFile
echo "::debug::Version PR created to ${DEVELOPMENT_BRANCH}"

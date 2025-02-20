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
#   $WORKFLOW - name of workflow which caused this to be called
#   $TRIGGER_SHA - sha commit which triggered the workflow
#   $DESTINATION_BRANCH - branch to create PR to
#   $ASSIGNEE - github id of person to assign PR to
#   $LABEL - label to apply to PR

# Validate required environment vars
if [[ ! -v WORKFLOW ]]; then
  echo "::error::WORKFLOW not specified"
  exit 1
elif [[ ! -v TRIGGER_SHA ]]; then
  echo "::error::TRIGGER_SHA not specified"
  exit 1
elif [[ ! -v DESTINATION_BRANCH ]]; then
  echo "::error::DESTINATION_BRANCH not specified"
  exit 1
elif [[ ! -v ASSIGNEE ]]; then
  echo "::error::ASSIGNEE not specified"
  exit 1
elif [[ ! -v LABEL ]]; then
  echo "::error::LABEL not specified"
  exit 1
elif [[ ! -v GITHUB_ACTOR ]]; then
  echo "::error::GITHUB_ACTOR not specified"
  exit 1
elif [[ ! -v ACTOR_EMAIL ]]; then
  echo "::error::ACTOR_EMAIL not specified"
  exit 1
elif [[ ! -v GITHUB_RUN_ID ]]; then
  echo "::error::GITHUB_RUN_ID not specified"
  exit 1
elif [[ ! -v REPO ]]; then
  echo "::error::REPO not specified"
  exit 1
fi

REPO=$(gh repo view --json nameWithOwner --jq '.nameWithOwner')
EXISTING_PR=$(gh pr list --state open --base "${DESTINATION_BRANCH}" --label "${LABEL}" --json headRefName,number --jq ".[] | select(.headRefName | contains(\"ci/fix/${TRIGGER_SHA}\") ) | .number")
ACTIONS_URL=$(gh api /repos/${REPO}/actions/runs/${GITHUB_RUN_ID} --jq '.html_url')
HEAD_REF_NAME="ci/fix/${TRIGGER_SHA}"

if [ -z "$EXISTING_PR" ]; then
  echo "::debug::No existing PR found for ${TRIGGER_SHA}"
  git config user.name "${GITHUB_ACTOR}"
  git config user.email "${ACTOR_EMAIL}"
  # Create a markdown file which will act as the body.
  bodyFile=prBodyFormat.md
  echo "# CI: Workflow failures: ${WORKFLOW}" >>$bodyFile
  echo "" >>$bodyFile
  echo "The [workflow](${ACTIONS_URL}) indicates failure, or requires attention. Please investigate." >>$bodyFile
  echo "" >>$bodyFile
  echo "## Details" >>$bodyFile
  echo "" >>$bodyFile
  echo "- Created by workflow: *${WORKFLOW}*" >>$bodyFile
  echo "- Triggered by \`${TRIGGER_SHA}\` commit" >>$bodyFile
  echo "" >>$bodyFile
  echo "" >>$bodyFile
  echo "> Actively maintained by the @Monotype/driverpdldev team." >>$bodyFile


  # Create a branch
  git checkout "${DESTINATION_BRANCH}"
  git checkout -b "${HEAD_REF_NAME}"
  git commit --allow-empty -m "CI: Workflow failure - ${WORKFLOW} - ${TRIGGER_SHA}"
  git push --set-upstream origin "${HEAD_REF_NAME}"

  # Create two PRs, one to the primary branch, and one to the development branch.
  gh pr create --base "${DESTINATION_BRANCH}" --assignee "${ASSIGNEE}" --title "Workflow failure - ${DESTINATION_BRANCH} - ${TRIGGER_SHA}" --label "${LABEL}" --body-file $bodyFile
  echo "::debug::Workflow fix PR created to ${DESTINATION_BRANCH} for ${TRIGGER_SHA}, using branch ${HEAD_REF_NAME}"
else
  echo "::debug::Existing PR found for ${TRIGGER_SHA}: ${EXISTING_PR}"
  exit 0
fi

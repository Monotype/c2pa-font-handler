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

# Brief: Updates the copyright year of all modified files to include the current
#        year, if a copyright string is found anywhere in the file.  If no
#        copyright string is found, will do nothing.  Expects the years to be
#        in order, from earliest to latest.
#
# Exceptions:
#   - .rs files - will add a new copyright string if one isn't encountered
#   - .github/workflows/* files: won't update these copyrights, as updating an
#                                active workflow file causes the commit to fail
#
# Examples (assuming current year is 2024):
#   - "2023" becomes "2023-2024"
#   - "2020-2023" becomes "2020-2024"
#   - "2020, 2022" becomes "2020, 2022, 2024"
#   - "2020-2021" becomes "2020-2021, 2024"
#
# Expects:
#   $BASE_BRANCH - base branch used to compare against to determine files which
#                  were modified.

# Fail the script if any commands within fail.
set -o errexit
# Generate error report on failure.
ErrorReport() { echo "::error::$0: Failed on line $1: ${BASH_COMMAND}"; }
trap 'ErrorReport $LINENO' ERR

# Validate required environment vars
if [[ ! -v BASE_BRANCH ]]
then
  echo "::error::BASE_BRANCH not specified"
  exit 1
fi

# Find the commit where the current branch diverged from the base branch; this
# allows us to get all changes in this branch even if the base branch has moved
# forward.
mergeBase=$(git merge-base HEAD "$BASE_BRANCH")

# Get all files which were modified as compared to the base branch.
changedFiles=$(git diff --cached --name-only --diff-filter=d "$mergeBase")

for file in $changedFiles
do
  # Make sure file exists.
  if ! [[ -f $file ]]
  then
    echo "::debug::File not found: $file"
    continue
  fi

  # Avoid updating copyrights in workflow files, if you end up editing a
  # workflow that's actively being used, it'll fail to commit.
  if [[ $file =~ \.github\/workflows\/ ]]
  then
    echo "::debug::Skipping workflow file: $file"
    continue
  fi

  # Regex to capture entire string of year specifiers.
  yearRegex="[0-9]{4}"
  yearDelimiter=",[[:blank:]]*"
  allYearsRegex="[[:blank:]]*((($yearRegex-$yearRegex($yearDelimiter)?)|($yearRegex($yearDelimiter)?))+)[[:blank:]]*"

  # New Rust copyright (also python scripts, yml files, natvis)
  newCopyrightRegex="Copyright${allYearsRegex}Monotype Imaging Inc\."
  # Old UFST copyright
  oldCopyrightRegex="Copyright \(C\)${allYearsRegex}Monotype Imaging Inc\."
  # At symbol copyright (used in bash scripts, jenkins, batch, docker, groovy)
  atSymbolCopyrightRegex="@copyright${allYearsRegex}Monotype Imaging Inc\."

  # Grab current year.
  currentYear=$(date +"%Y")

  # Find the copyright line.
  copyrightLineNumber=1
  copyrightLine=
  while IFS= read -r line
  do
    if [[ ($line =~ $newCopyrightRegex) || ($line =~ $oldCopyrightRegex) || ($line =~ $atSymbolCopyrightRegex)]]
      then
        copyrightLine=$line
        # Grab the string including all years.
        yearsString="${BASH_REMATCH[1]}"
        break
    fi
    ((copyrightLineNumber++))
  done < "$file"

  # If we have no copyright, done.
  if [[ -z $copyrightLine ]]
  then
    # If we're a rust file, it's safe to add in a new copyright string.
    if [[ $file == *".rs" ]]
    then
      newCopyright="\/\/ Copyright $currentYear Monotype Imaging Inc."
      sed -i "1i$newCopyright" "$file"
      echo "::debug::New copyright added to $file"
    else
      echo "::debug::No copyright string found, skipping: $file"
    fi
    continue
  fi

  # Grab all text before and after the years.
  beforeYears="${copyrightLine/[0-9]*}"
  afterYears="${copyrightLine##*[0-9]}"

  # Trim trailing whitespace of before.
  beforeYears="${beforeYears%"${beforeYears##*[![:space:]]}"}"
  # Trim leading whitespace of after.
  afterYears="${afterYears#"${afterYears%%[![:space:]]*}"}"

  # Remove all whitespace from years string.
  yearsString=$(echo "$yearsString" | sed "s/[[:blank:]]//g")

  # Split year specifiers into an array.
  IFS=',' read -r -a yearSpecifiers <<< "$yearsString"
  # Grab the last year specifier in the array.
  lastYearSpecifier=${yearSpecifiers[-1]}

  # If the last year group is the current year, done.
  if [[ $lastYearSpecifier == "$currentYear" ]]
  then
    echo "::debug::Current year included in copyright, skipping: $file"
    continue
  fi

  # If the last year specifier is a range..
  if [[ $lastYearSpecifier =~ "-" ]]
  then
    # Split range into an array
    IFS='-' read -r -a range <<< "$lastYearSpecifier"
    startYear=${range[0]}
    endYear=${range[1]}
    # If end year is the current year, done.
    if [[ $endYear == "$currentYear" ]]
    then
      echo "::debug::Current year included in copyright, skipping: $file"
      continue
    # If end year is the previous year, expand range to include current year.
    elif [[ $endYear == $((currentYear-1)) ]]
    then
      # Replace the end year with the current year.
      yearSpecifiers[-1]="$startYear-$currentYear"
    # Otherwise, just add current year to list.
    else
      yearSpecifiers+=("$currentYear")
    fi
  # Otherwise, dealing with a single year.
  else
    # If last year is the previous year, make a range.
    if [[ $lastYearSpecifier == $((currentYear-1)) ]]
    then
      # Replace the last year with a range including the current year.
      yearSpecifiers[-1]="$lastYearSpecifier-$currentYear"
    # If the last year wasn't the previous year, add the current year.
    else
      yearSpecifiers+=("$currentYear")
    fi
  fi

  # Build up new copyright string with fixed year specifiers.
  newCopyright="$beforeYears "
  for i in "${yearSpecifiers[@]}"
  do
    newCopyright+="$i"
    # Add comma if not last year
    if [[ $i != "${yearSpecifiers[-1]}" ]]
    then
      newCopyright+=", "
    fi
  done
  newCopyright+=" $afterYears"

  # Escape slashes.
  newCopyright=$(echo "$newCopyright" | sed 's/\//\\\//g')
  # Add new copyright into file.
  sed -i "$copyrightLineNumber s/.*/$newCopyright/" "$file"
  echo "::debug::Copyright updated: $file"
done
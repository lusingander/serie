#!/bin/bash
#
# Generate test git repository with realistic branch patterns
#
# Usage: ./generate_test_repo.sh [path] [commit_count]
#        ./generate_test_repo.sh /tmp/my-repo 5000
#
# Creates a repository with:
#   - Main branch with linear history
#   - feature/xxx and bugfix/xxx branches forking from main
#   - Each branch lives 2-8 commits before merging back
#   - Merges use --no-ff to preserve merge commits
#   - Max 5 concurrent branches
#   - Semver tags (v0.1.0, v0.2.3, ...)
#   - Multiple authors with realistic dates spread over 2 years
#
# Resulting graph looks like typical "christmas tree":
#
#   *   merge feature/auth-42
#   |\
#   | * feat(auth): add auth logic
#   | * feat(auth): implement auth logic
#   |/
#   *   fix: merge bugfix/timeout-15
#   |\
#   | * fix(api): fix api logic
#   |/
#   * feat(cache): update cache logic
#   * chore: initial commit
#

set -e

REPO_DIR="${1:-/tmp/test-repo-10k}"
COMMIT_COUNT="${2:-10000}"

rm -rf "$REPO_DIR"
mkdir -p "$REPO_DIR"
cd "$REPO_DIR"
git init
git config user.email "dev@example.com"
git config user.name "Developer"

# Arrays for random content
AUTHORS=(
    "Alice Smith:alice@example.com"
    "Bob Johnson:bob@example.com"
    "Charlie Brown:charlie@example.com"
    "Diana Prince:diana@example.com"
    "Eve Wilson:eve@example.com"
)

PREFIXES=("feat" "fix" "refactor" "docs" "test" "chore" "perf")

FEATURES=(
    "auth" "api" "database" "cache" "search" "upload" "export"
    "notifications" "settings" "dashboard" "reports" "billing"
    "users" "permissions" "logging" "metrics" "config" "cli"
)

BUGS=(
    "login-redirect" "null-pointer" "memory-leak" "timeout" "encoding"
    "validation" "race-condition" "deadlock" "overflow" "parsing"
    "connection-drop" "cache-invalidation" "timezone" "locale"
)

ACTIONS=("add" "update" "fix" "improve" "implement" "refactor" "optimize")

random_element() {
    local arr=("$@")
    echo "${arr[$RANDOM % ${#arr[@]}]}"
}

random_author() {
    local info=$(random_element "${AUTHORS[@]}")
    echo "$info"
}

generate_commit_message() {
    local prefix=$(random_element "${PREFIXES[@]}")
    local feature=$(random_element "${FEATURES[@]}")
    local action=$(random_element "${ACTIONS[@]}")
    echo "$prefix($feature): $action ${feature} logic"
}

generate_file_content() {
    echo "// Updated at $(date +%s%N)"
    echo "// Random: $RANDOM"
    for i in {1..10}; do
        echo "fn func_$RANDOM() { /* ... */ }"
    done
}

make_commit() {
    local message="$1"
    local author_info="$2"
    local commit_date="$3"

    local author_name="${author_info%%:*}"
    local author_email="${author_info##*:}"

    local component=$(random_element "${FEATURES[@]}")
    local file_path="src/${component}.rs"
    mkdir -p "$(dirname "$file_path")"
    generate_file_content > "$file_path"
    git add "$file_path"

    GIT_AUTHOR_NAME="$author_name" \
    GIT_AUTHOR_EMAIL="$author_email" \
    GIT_AUTHOR_DATE="$commit_date" \
    GIT_COMMITTER_NAME="$author_name" \
    GIT_COMMITTER_EMAIL="$author_email" \
    GIT_COMMITTER_DATE="$commit_date" \
    git commit -m "$message" --quiet 2>/dev/null || true
}

echo "Generating $COMMIT_COUNT commits in $REPO_DIR..."

# Initial commit
mkdir -p src
echo "# Test Repository" > README.md
echo "fn main() {}" > src/main.rs
git add .
git commit -m "chore: initial commit" --quiet

start_time=$(date +%s)
base_date=$(date -d "2023-01-01" +%s 2>/dev/null || date -j -f "%Y-%m-%d" "2023-01-01" +%s 2>/dev/null || echo "1672531200")
seconds_per_commit=$(( 730 * 24 * 3600 / COMMIT_COUNT ))  # spread over 2 years

commit_num=0
active_branches=()
next_branch_at=$((RANDOM % 20 + 5))
branch_counter=0
version_major=0
version_minor=0
version_patch=0

while [ $commit_num -lt $COMMIT_COUNT ]; do
    # Progress
    if [ $((commit_num % 500)) -eq 0 ] && [ $commit_num -gt 0 ]; then
        elapsed=$(($(date +%s) - start_time))
        rate=$((commit_num / (elapsed + 1)))
        echo "Progress: $commit_num/$COMMIT_COUNT ($rate/sec)"
    fi

    current_date=$((base_date + commit_num * seconds_per_commit + RANDOM % 3600))
    commit_date=$(date -d "@$current_date" --iso-8601=seconds 2>/dev/null || date -r "$current_date" +%Y-%m-%dT%H:%M:%S 2>/dev/null || echo "2023-06-15T12:00:00")
    author=$(random_author)

    # Decide: work on main or create/continue feature branch
    if [ ${#active_branches[@]} -eq 0 ] || [ $((RANDOM % 3)) -eq 0 ]; then
        # Work on main
        git checkout main --quiet 2>/dev/null || git checkout -b main --quiet

        # Maybe start a new branch
        if [ $commit_num -ge $next_branch_at ] && [ ${#active_branches[@]} -lt 5 ]; then
            branch_counter=$((branch_counter + 1))

            # Feature or bugfix?
            if [ $((RANDOM % 3)) -eq 0 ]; then
                bug=$(random_element "${BUGS[@]}")
                branch_name="bugfix/${bug}-${branch_counter}"
                branch_type="bugfix"
            else
                feature=$(random_element "${FEATURES[@]}")
                branch_name="feature/${feature}-${branch_counter}"
                branch_type="feature"
            fi

            git checkout -b "$branch_name" --quiet
            active_branches+=("$branch_name:$branch_type:1")
            next_branch_at=$((commit_num + RANDOM % 30 + 10))

            make_commit "$(generate_commit_message)" "$author" "$commit_date"
            commit_num=$((commit_num + 1))
        else
            # Regular main commit
            make_commit "$(generate_commit_message)" "$author" "$commit_date"
            commit_num=$((commit_num + 1))

            # Maybe tag a release
            if [ $((RANDOM % 100)) -eq 0 ]; then
                version_patch=$((version_patch + 1))
                if [ $version_patch -ge 10 ]; then
                    version_patch=0
                    version_minor=$((version_minor + 1))
                fi
                if [ $version_minor -ge 10 ]; then
                    version_minor=0
                    version_major=$((version_major + 1))
                fi
                git tag "v${version_major}.${version_minor}.${version_patch}" 2>/dev/null || true
            fi
        fi
    else
        # Work on existing branch
        idx=$((RANDOM % ${#active_branches[@]}))
        branch_info="${active_branches[$idx]}"
        branch_name="${branch_info%%:*}"
        rest="${branch_info#*:}"
        branch_type="${rest%%:*}"
        branch_commits="${rest##*:}"

        git checkout "$branch_name" --quiet 2>/dev/null || continue

        # Add commit to branch
        make_commit "$(generate_commit_message)" "$author" "$commit_date"
        commit_num=$((commit_num + 1))
        branch_commits=$((branch_commits + 1))

        # Update branch info
        active_branches[$idx]="$branch_name:$branch_type:$branch_commits"

        # Maybe merge back to main (after 2-8 commits)
        if [ $branch_commits -ge $((RANDOM % 7 + 2)) ]; then
            git checkout main --quiet

            # Merge with descriptive message
            if [ "$branch_type" = "bugfix" ]; then
                merge_msg="fix: merge $branch_name"
            else
                merge_msg="feat: merge $branch_name"
            fi

            git merge --no-ff "$branch_name" -m "$merge_msg" --quiet 2>/dev/null || {
                git merge --abort 2>/dev/null || true
                git checkout main --quiet
            }

            git branch -d "$branch_name" --quiet 2>/dev/null || true

            # Remove from active branches
            unset 'active_branches[$idx]'
            active_branches=("${active_branches[@]}")
        fi
    fi
done

# Merge remaining branches
git checkout main --quiet 2>/dev/null || true
for branch_info in "${active_branches[@]}"; do
    branch_name="${branch_info%%:*}"
    if git show-ref --verify --quiet "refs/heads/$branch_name" 2>/dev/null; then
        git merge --no-ff "$branch_name" -m "feat: merge $branch_name" --quiet 2>/dev/null || git merge --abort 2>/dev/null || true
        git branch -d "$branch_name" --quiet 2>/dev/null || true
    fi
done

end_time=$(date +%s)
elapsed=$((end_time - start_time))

echo ""
echo "Done!"
echo "Repository: $REPO_DIR"
echo "Commits: $(git rev-list --count HEAD)"
echo "Tags: $(git tag | wc -l)"
echo "Time: ${elapsed}s"
echo ""
echo "Test with: serie $REPO_DIR"

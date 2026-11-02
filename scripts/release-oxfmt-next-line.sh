#!/usr/bin/env bash
# Publish oxfmt-next-line from the packages CI built for the current commit, then push the release tag.
# CI (.github/workflows/release_oxfmt_next_line.yml) builds the 19 native bindings and uploads them
# as the `oxfmt-next-line-npm` artifact; this script publishes them with the local npm login.
set -euo pipefail

repo=motlin/oxc-next-line
workflow=release_oxfmt_next_line.yml
artifact=oxfmt-next-line-npm

if [[ "$(git branch --show-current)" != main || -n "$(git status --porcelain --untracked-files=no)" ]]; then
    echo "Release requires a clean main branch. Commit the version change first." >&2
    exit 1
fi

release_commit=$(git rev-parse HEAD)
version=$(node -p 'require("./npm/oxfmt/package.json").version')
release_tag="oxfmt-next-line@${version}"
git check-ref-format "refs/tags/$release_tag"
git fetch origin main
if [[ "$(git rev-parse FETCH_HEAD)" != "$release_commit" ]]; then
    echo "Push main to origin before releasing." >&2
    exit 1
fi
remote_tag=$(git ls-remote --tags origin "refs/tags/$release_tag")
if git show-ref --verify --quiet "refs/tags/$release_tag" || [[ -n "$remote_tag" ]]; then
    echo "Release tag $release_tag already exists." >&2
    exit 1
fi

run_id=$(gh run list --repo "$repo" --workflow "$workflow" --commit "$release_commit" --status success \
    --json databaseId --jq '.[0].databaseId // empty')
if [[ -z "$run_id" ]]; then
    echo "No successful $workflow run for $release_commit." >&2
    echo "Wait for the run started by the version bump, or start one: gh workflow run $workflow --repo $repo --ref main" >&2
    exit 1
fi

work=$(mktemp -d)
trap 'rm -r "$work"' EXIT
gh run download "$run_id" --repo "$repo" --name "$artifact" --dir "$work"

built_version=$(node -p "require('$work/oxfmt/package.json').version")
if [[ "$built_version" != "$version" ]]; then
    echo "Artifact from run $run_id is version $built_version, expected $version." >&2
    exit 1
fi
binding_count=$(find "$work/oxfmt-release" -mindepth 1 -maxdepth 1 -type d | wc -l | tr -d ' ')
optional_count=$(node -p "Object.keys(require('$work/oxfmt/package.json').optionalDependencies ?? {}).length")
if [[ "$binding_count" != 19 || "$optional_count" != 19 ]]; then
    echo "Expected 19 binding packages and 19 optionalDependencies, found $binding_count and $optional_count." >&2
    exit 1
fi

publish_if_needed() {
    local dir=$1 spec
    spec=$(node -p "const p = require('$dir/package.json'); p.name + '@' + p.version")
    if [[ "$(npm view "$spec" version 2>/dev/null)" == "${spec##*@}" ]]; then
        echo "Skipping $spec, already published."
        return
    fi
    npm publish "$dir" --access public
}

for dir in "$work"/oxfmt-release/*/; do
    publish_if_needed "${dir%/}"
done
# npm publish can return before the bindings are installable, and the main package depends on them.
node .github/scripts/wait-for-npm-packages.mjs "$work"/oxfmt-release/*
publish_if_needed "$work/oxfmt"

# Preserve the published commit locally even if the remote push fails.
git tag --annotate "$release_tag" "$release_commit" --message "Release $release_tag."
if ! git push origin "refs/tags/$release_tag"; then
    echo "npm publication succeeded. Retry: git push origin refs/tags/$release_tag" >&2
    exit 1
fi

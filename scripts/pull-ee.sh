#!/usr/bin/env bash
# Sync the local Enterprise overlay (ee/) from the private EE repository.
#
# Usage:
#   ./scripts/pull-ee.sh
#
# Optional overrides:
#   EE_REPO_URL=git@github.com:AgentKern/agentkern-ee.git
#   EE_BRANCH=main

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
EE_DIR="${ROOT_DIR}/ee"

EE_REPO_URL="${EE_REPO_URL:-git@github.com:AgentKern/agentkern-ee.git}"
EE_BRANCH="${EE_BRANCH:-main}"

echo "🔄 Syncing Enterprise overlay"
echo "   repo:   ${EE_REPO_URL}"
echo "   branch: ${EE_BRANCH}"
echo "   target: ${EE_DIR}"

if [[ -d "${EE_DIR}" ]]; then
    if [[ ! -d "${EE_DIR}/.git" ]]; then
        echo "❌ ${EE_DIR} exists but is not a git repository."
        echo "   Move/rename it, then rerun this script."
        exit 1
    fi

    if [[ -n "$(git -C "${EE_DIR}" status --porcelain)" ]]; then
        echo "❌ ${EE_DIR} has uncommitted changes."
        echo "   Commit/stash/discard changes before syncing."
        exit 1
    fi

    if git -C "${EE_DIR}" remote get-url origin >/dev/null 2>&1; then
        CURRENT_ORIGIN="$(git -C "${EE_DIR}" remote get-url origin)"
        if [[ "${CURRENT_ORIGIN}" != "${EE_REPO_URL}" ]]; then
            echo "⚠️  origin currently points to: ${CURRENT_ORIGIN}"
            echo "   updating origin to:          ${EE_REPO_URL}"
            git -C "${EE_DIR}" remote set-url origin "${EE_REPO_URL}"
        fi
    else
        git -C "${EE_DIR}" remote add origin "${EE_REPO_URL}"
    fi

    git -C "${EE_DIR}" fetch origin "${EE_BRANCH}"

    if git -C "${EE_DIR}" show-ref --verify --quiet "refs/heads/${EE_BRANCH}"; then
        git -C "${EE_DIR}" checkout "${EE_BRANCH}"
    else
        git -C "${EE_DIR}" checkout -b "${EE_BRANCH}" --track "origin/${EE_BRANCH}"
    fi

    git -C "${EE_DIR}" pull --ff-only origin "${EE_BRANCH}"
else
    git clone --branch "${EE_BRANCH}" "${EE_REPO_URL}" "${EE_DIR}"
fi

echo "✅ Enterprise overlay is up to date."

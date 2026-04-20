#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

required_files=(
  "README.md"
  "docs/OSS_SETUP.md"
  "docs/OSS_CAPABILITY_MATRIX.md"
  "docs/governance/OSS_PRODUCT_BOUNDARY.md"
  "docs/governance/CI_POLICY.md"
  "docs/governance/OSS_TRANSITION_STATUS.md"
  "docs/reference/POSITIONING_VS_ALTERNATIVES.md"
  "docs/reference/API_REFERENCE.md"
  "apps/server/src/main.rs"
)

for file in "${required_files[@]}"; do
  if [[ ! -f "$file" ]]; then
    echo "Missing required file: $file"
    exit 1
  fi
done

if [[ ! -x "scripts/verify-ci-policy.sh" ]]; then
  echo "scripts/verify-ci-policy.sh must exist and be executable."
  exit 1
fi

if ! grep -q "edition_mode" "apps/server/src/main.rs"; then
  echo "Missing edition_mode in server health contract."
  exit 1
fi

if ! grep -q "quarantined_pillars" "apps/server/src/main.rs"; then
  echo "Missing quarantined_pillars in server health contract."
  exit 1
fi

if ! grep -Eq "treasury.*quarantined" "apps/server/src/main.rs"; then
  echo "Missing Treasury quarantined runtime marker in server."
  exit 1
fi

if ! grep -q "OSS_CAPABILITY_MATRIX.md" "README.md"; then
  echo "README.md must link OSS_CAPABILITY_MATRIX.md."
  exit 1
fi

if ! grep -q "OSS_CAPABILITY_MATRIX.md" "docs/OSS_SETUP.md"; then
  echo "docs/OSS_SETUP.md must link OSS_CAPABILITY_MATRIX.md."
  exit 1
fi

if ! grep -Eq "Treasury.*Quarantined|/api/v1/treasury/.+Quarantined|quarantined in default OSS server mode" "docs/OSS_CAPABILITY_MATRIX.md" "docs/OSS_SETUP.md" "docs/reference/API_REFERENCE.md"; then
  echo "Treasury quarantine status is not consistently documented."
  exit 1
fi

echo "OSS capability consistency checks passed."

#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

python3 - <<'PY'
from pathlib import Path
import re
import sys

workflow_dir = Path(".github/workflows")
allowed = {
    ("ci.yml", "Run coverage"),
    ("security.yml", "SonarQube Scan (Rust)"),
}

violations: list[str] = []
found: set[tuple[str, str]] = set()

for file_path in sorted(workflow_dir.glob("*.yml")):
    lines = file_path.read_text(encoding="utf-8").splitlines()
    current_step = None
    step_start = 0

    for idx, line in enumerate(lines):
        step_match = re.match(r"\s*-\s+name:\s+(.+)\s*$", line)
        if step_match:
            current_step = step_match.group(1).strip()
            step_start = idx
            continue

        if re.search(r"\bcontinue-on-error:\s*true\b", line):
            if not current_step:
                violations.append(
                    f"{file_path.name}:{idx+1}: continue-on-error outside named step"
                )
                continue

            found.add((file_path.name, current_step))
            if (file_path.name, current_step) not in allowed:
                violations.append(
                    f"{file_path.name}:{idx+1}: unauthorized continue-on-error in step '{current_step}'"
                )
                continue

            context = "\n".join(lines[step_start:idx + 1])
            if "CI policy: docs/governance/CI_POLICY.md" not in context:
                violations.append(
                    f"{file_path.name}:{idx+1}: missing CI policy comment in step '{current_step}'"
                )

missing_allowed = sorted(allowed - found)
if missing_allowed:
    for workflow_name, step_name in missing_allowed:
        violations.append(
            f"{workflow_name}: expected advisory step '{step_name}' not found"
        )

if violations:
    print("CI policy validation failed:")
    for violation in violations:
        print(f" - {violation}")
    sys.exit(1)

print("CI policy validation passed.")
PY

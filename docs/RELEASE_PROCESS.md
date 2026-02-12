# Release Process

This document describes the process for creating releases of AgentKern.

---

## 🏷️ Versioning

AgentKern follows [Semantic Versioning](https://semver.org/):

- **MAJOR** (1.0.0): Breaking changes
- **MINOR** (0.1.0): New features, backward compatible
- **PATCH** (0.0.1): Bug fixes, backward compatible

### Pre-release Versions

- **Alpha**: `0.1.0-alpha.1` - Early development
- **Beta**: `0.1.0-beta.1` - Feature complete, testing
- **RC**: `0.1.0-rc.1` - Release candidate

---

## 📋 Release Checklist

### Before Release

- [ ] All tests passing: `cargo test --workspace`
- [ ] Linting clean: `cargo clippy --workspace -- -D warnings`
- [ ] Formatting: `cargo fmt --all --check`
- [ ] Documentation updated
- [ ] CHANGELOG.md updated with release notes
- [ ] Version bumped in `Cargo.toml` workspace
- [ ] Version bumped in `package.json` (if applicable)
- [ ] Security audit: `cargo audit`
- [ ] Performance benchmarks run (if applicable)

### Release Steps

1. **Update CHANGELOG.md**
   ```markdown
   ## [0.1.0] - 2026-01-03
   
   ### Added
   - Feature X
   - Feature Y
   
   ### Changed
   - Improvement Z
   
   ### Fixed
   - Bug fix A
   ```

2. **Update Version**
   ```bash
   # In Cargo.toml workspace
   version = "0.1.0"
   
   # In package.json (if applicable)
   "version": "0.1.0"
   ```

3. **Create Release Branch**
   ```bash
   git checkout -b release/v0.1.0
   git add .
   git commit -m "chore: prepare release v0.1.0"
   git push origin release/v0.1.0
   ```

4. **Create Tag**
   ```bash
   git tag -a v0.1.0 -m "Release v0.1.0"
   git push origin v0.1.0
   ```

5. **GitHub Release**
   - The `.github/workflows/release.yml` will automatically:
     - Build binaries for multiple platforms
     - Create GitHub release
     - Upload artifacts
     - Generate release notes

6. **Merge to Main**
   ```bash
   git checkout main
   git merge release/v0.1.0
   git push origin main
   ```

7. **Post-Release**
   - Update version to next dev version in `Cargo.toml`
   - Create "Unreleased" section in CHANGELOG.md
   - Announce release (GitHub Discussions, etc.)

---

## 🚀 Automated Release (GitHub Actions)

The release workflow (`.github/workflows/release.yml`) automatically:

1. **Builds** binaries for:
   - Linux (x86_64)
   - macOS (x86_64)
   - Windows (x86_64)

2. **Creates** GitHub release with:
   - Release notes from CHANGELOG
   - Binary artifacts
   - Source code archive

3. **Publishes** (optional) to crates.io if `CRATES_IO_TOKEN` is set

### Triggering Release

**Option 1: Tag Push**
```bash
git tag -a v0.1.0 -m "Release v0.1.0"
git push origin v0.1.0
```

**Option 2: Manual Workflow**
- Go to Actions → Release → Run workflow
- Enter version number
- Workflow will create tag and release

---

## 📦 Publishing to Package Registries

### crates.io (Rust)

```bash
# Publish individual packages
cargo publish -p agentkern-gate
cargo publish -p agentkern-identity
# ... etc

# Requires: CRATES_IO_TOKEN secret in GitHub
```

### npm (Node.js SDK)

```bash
cd sdks/node
npm version patch  # or minor, major
npm publish
```

### PyPI (Python SDK)

```bash
cd sdks/python
python -m build
twine upload dist/*
```

---

## 🔍 Release Verification

After release, verify:

- [ ] GitHub release created
- [ ] Binaries downloadable
- [ ] Release notes accurate
- [ ] Version tags correct
- [ ] Documentation links work
- [ ] Installation instructions work

---

## 📝 Release Notes Template

```markdown
# AgentKern v0.1.0

## What's New

### Major Features
- Feature 1
- Feature 2

### Improvements
- Improvement 1
- Improvement 2

### Bug Fixes
- Fix 1
- Fix 2

### Breaking Changes
- ⚠️ Breaking change 1 (migration guide: [link])

## Installation

\`\`\`bash
# From source
cargo install --git https://github.com/AgentKern/agentkern --tag v0.1.0 agentkern-server

# Or download binaries from assets below
\`\`\`

## Full Changelog

See [CHANGELOG.md](https://github.com/AgentKern/agentkern/blob/v0.1.0/CHANGELOG.md)
```

---

## 🐛 Hotfix Releases

For critical bug fixes:

1. Create hotfix branch from latest release tag
2. Apply fix
3. Bump patch version
4. Create hotfix release
5. Merge back to main

---

## 📊 Release Metrics

Track for each release:
- Number of commits
- Lines changed
- Tests added
- Contributors
- Issues closed

---

**Questions?** Open an issue or contact maintainers.

# Contributing to AgentKern

Thank you for your interest in contributing to AgentKern! This document provides guidelines and instructions for contributing.

---

## 🎯 How to Contribute

### Reporting Bugs

1. **Check existing issues**: Search [GitHub Issues](https://github.com/AgentKern/agentkern/issues) to see if the bug is already reported.
2. **Create a new issue**: If not found, create a new issue with:
   - Clear title and description
   - Steps to reproduce
   - Expected vs actual behavior
   - Environment details (OS, Rust version, etc.)
   - Relevant logs/error messages

### Suggesting Features

1. **Check existing issues**: Search for similar feature requests.
2. **Create a feature request**: Use the "Feature Request" template with:
   - Clear description of the feature
   - Use case and motivation
   - Proposed implementation approach (if applicable)

### Code Contributions

1. **Fork the repository**
2. **Create a feature branch**: `git checkout -b feature/your-feature-name`
3. **Make your changes**
4. **Write/update tests**: Ensure all tests pass
5. **Run linters**: `cargo fmt --all && cargo clippy --workspace -- -D warnings`
6. **Commit with clear messages**: Follow [Conventional Commits](https://www.conventionalcommits.org/)
7. **Push and create a Pull Request**

---

## 🏗️ Development Setup

### Prerequisites

- **Rust**: 1.92+ (stable)
- **Node.js**: 24+ (for SDKs and playground)
- **pnpm**: 9+ (package manager)
- **PostgreSQL**: 17+ (for integration tests)
- **Docker**: (optional, for containerized testing)

### Initial Setup

```bash
# Clone the repository
git clone https://github.com/AgentKern/agentkern.git
cd agentkern

# Install Rust dependencies
cargo build --workspace

# Install Node.js dependencies
pnpm install

# Run tests
cargo test --workspace
```

### Project Structure

```
agentkern/
├── apps/
│   ├── server/          # Unified Rust HTTP server
│   └── playground/      # TypeScript frontend
├── packages/
│   ├── pillars/        # Six core pillars (Rust)
│   └── foundation/     # Shared infrastructure
├── sdks/               # Client libraries
├── docs/               # Documentation
└── tests/              # Integration tests
```

---

## 📝 Code Standards

### Rust

- **Formatting**: Use `cargo fmt --all`
- **Linting**: Must pass `cargo clippy --workspace -- -D warnings`
- **Tests**: All tests must pass: `cargo test --workspace`
- **Documentation**: Public APIs must have doc comments
- **Error Handling**: Use `Result<T, E>` instead of `unwrap()` in production code

### TypeScript/JavaScript

- **Formatting**: Use Prettier (configured in repo)
- **Linting**: Must pass ESLint checks
- **Type Safety**: Prefer TypeScript, avoid `any`

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types**: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

**Examples**:
- `feat(gate): add prompt injection detection`
- `fix(identity): resolve WebAuthn credential parsing`
- `docs: update quickstart guide`

---

## 🧪 Testing

### Running Tests

```bash
# All tests
cargo test --workspace

# Specific pillar
cargo test -p agentkern-gate

# With output
cargo test --workspace -- --nocapture

# Integration tests
cargo test --test integration
```

### Writing Tests

- **Unit tests**: In `src/` files with `#[cfg(test)]`
- **Integration tests**: In `tests/` directory
- **Coverage**: Aim for >80% coverage on new code

---

## 🔍 Pull Request Process

1. **Update CHANGELOG.md**: Add entry under "Unreleased"
2. **Update documentation**: If adding features, update relevant docs
3. **Ensure CI passes**: All checks must be green
4. **Request review**: Tag relevant maintainers
5. **Address feedback**: Respond to review comments
6. **Squash commits**: Before merging (if requested)

### PR Checklist

- [ ] Code follows style guidelines
- [ ] Tests added/updated and passing
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] No breaking changes (or documented if intentional)
- [ ] CI checks passing

---

## 🏛️ Architecture Decisions

For significant changes, consider creating an ADR (Architecture Decision Record):

1. Create `docs/governance/adr/ADR-XXX-your-decision.md`
2. Follow the ADR template
3. Get approval from maintainers

---

## 📚 Documentation

### Writing Documentation

- **Code comments**: Use Rust doc comments for public APIs
- **README files**: Keep package READMEs up to date
- **Guides**: Add to `docs/guides/` for how-to content
- **Specs**: Add to `docs/spec/` for design specifications

### Documentation Standards

- Clear, concise language
- Code examples where helpful
- Link to related docs
- Keep up to date with code changes

---

## 🚫 What NOT to Contribute

- **Enterprise features**: `ee/` directory is proprietary
- **Breaking changes**: Discuss in issues first
- **Large refactors**: Create an ADR first
- **Dependencies**: Get approval before adding new crates

---

## 🤝 Code of Conduct

- Be respectful and inclusive
- Welcome newcomers
- Focus on constructive feedback
- Follow the project's technical decisions

---

## 📞 Getting Help

- **GitHub Discussions**: For questions and discussions
- **GitHub Issues**: For bugs and feature requests
- **Discord**: (if available) For real-time chat

---

## 🙏 Recognition

Contributors will be:
- Listed in CONTRIBUTORS.md
- Mentioned in release notes (for significant contributions)
- Credited in the project documentation

---

**Thank you for contributing to AgentKern!** 🎉

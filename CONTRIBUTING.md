# Contributing to CAFramework

Thank you for your interest in contributing to **CAFramework**! We welcome contributions, bug fixes, documentation improvements, and architectural suggestions.

---

## 1. Development Workflow
1. Fork the repository and create your feature branch: `git checkout -b feat/my-new-feature`.
2. Ensure you have Rust (Edition 2024 / 1.85+) and Node.js (v20+) installed.
3. Install frontend dependencies:
   ```bash
   cd frontend && npm install
   ```
4. Run tests and verify formatting before submitting PR:
   ```bash
   cargo fmt --all
   cargo test --workspace
   cd frontend && npm run check && npm run build
   ```

---

## 2. Commit Message Convention
We follow Conventional Commits:
- `feat(scope): ...` for new features
- `fix(scope): ...` for bug fixes
- `docs(scope): ...` for documentation changes
- `refactor(scope): ...` for code refactoring
- `test(scope): ...` for test suites

---

## 3. Submitting Pull Requests
- Open a PR against the `main` branch.
- Clearly describe the problem solved or feature added.
- Link any relevant issues.

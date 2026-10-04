# Contributing to CAFramework

Thank you for your interest in contributing to CAFramework!

## Guidelines
1. **Branching & PRs**: Create feature/bugfix branches and submit pull requests.
2. **Quality Bars**:
   - Run `cargo test --workspace` to ensure all unit and architectural tests pass.
   - Run `cargo clippy --workspace --all-targets` without warnings.
   - Run `cargo run -p caf-xtask -- guard` to check architectural invariants and security ratchets.
   - Frontend changes must pass `npm test` and Svelte checks.
3. **Security**: Do not hardcode credentials, tokens, or PII. Review [SECURITY.md](SECURITY.md) for vulnerability reporting.

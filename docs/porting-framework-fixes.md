# Porting Framework Fixes to Derivative Apps

This guide covers how upstream improvements, security updates, and schema migrations in `caframework` can be cleanly ported to derivative applications created via `new-app`.

## 1. Upstream Git Remote Strategy
Add the framework repository as an upstream remote:
```bash
git remote add upstream https://github.com/cecepazhar/caframework.git
git fetch upstream
```

## 2. Reviewing Migrations
When upgrading `caf-core` or database migrations:
- Check `crates/caf-core/src/migrations/` for new version definitions.
- Inspect `docs/error-codes.md` for new error categories.

## 3. Synchronizing Frontend Stores & Components
- Compare `frontend/src/lib/components/` for UI layout and security component updates (TitleBar, PINPad, Lockscreen).

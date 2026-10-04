# ADR 0003: IPC Bindings and Generator Contract

**Status:** Accepted
**Context:** CAFramework passes data frequently between the Svelte frontend and the Tauri backend. Currently, `invoke("command")` calls use magic strings and untyped arguments, leading to bugs like renamed fields causing runtime silent failures (e.g., `never_again` bug, password changes, AI fields). Mismatches in snake_case vs camelCase serialization also cause errors.

**Decision:**
1. We will adopt the `ts-rs` crate to annotate and export strongly typed TypeScript interfaces for all Rust models that cross the IPC boundary.
2. We will extend `caf-xtask codegen` to use the `syn` crate. It will parse `crates/caf-app/src/commands.rs`, identify all `#[tauri::command]` functions, read their arguments and return types, and generate a 1:1 `commands.ts` file wrapping Tauri's `invoke()`.
3. We avoid `tauri-specta` because our CAFramework environment requires 100% control over the generated file layout without wrestling with unstable RC feature flags of third-party bundlers. The custom `syn` parsing xtask provides greater flexibility within our monorepo architecture.
4. All frontend files (`frontend/src/lib/api/*`) MUST import from `frontend/src/lib/generated/commands.ts` and not use `invoke()` directly.

**Consequences:**
- Types are strictly coupled. Any field name change in Rust instantly breaks `npm run check`, enforcing zero-drift validation.
- Commands use `camelCase` args on the TypeScript side automatically transpiling to Rust's `snake_case`.
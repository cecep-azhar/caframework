# CAFramework — Prompt Eksekusi Addendum v2 (25 September 2026)

> **Konteks:** hasil sesi brainstorming 25 Sep 2026 — 8 keputusan yang mengubah/menambah scope
> `caframework-execution-prompts-v1.md`. Baca dulu dokumen keputusan lengkapnya di project claude.ai
> CAFramework: **"keputusan-arsitektur-ekosistem-caframework"** untuk konteks kenapa tiap keputusan diambil.
> **Cara pakai sama seperti v1**: tiap prompt di bawah self-contained, paste ke sesi agent coding
> baru yang tidak punya konteks percakapan ini.
> **Bahasa**: instruksi prompt dalam Bahasa Inggris (standar prompt library), tapi setiap prompt
> secara eksplisit meminta agent lapor progres & hasil akhir dalam Bahasa Indonesia.
> **Prinsip umum semua prompt di bawah**: jangan klaim "selesai/aman" tanpa bukti (command +
> output nyata, bukan ringkasan). Kalau ada yang tidak jelas/ambigu, TANYA dulu sebelum eksekusi,
> jangan asumsikan.

---

## Urutan Prioritas (lihat task list untuk status live)

1. Android Keystore Setup — **blocking**, paling cepat, murni operasional
2. Privacy Policy — paralel, sebelum Fase 1 (newsletter) live
3. Landing Fase 0 — blocked by #1 (butuh keystore sebelum expose link APK)
4. Team Vault Key-Wrapping Design — sesi desain, bukan kode
5. Verifikasi kapasitas infra Ollama
6. GCC Licensing Backend (4a, updated) — blocked by #5
7. Licensing Client Rust (4b, updated) — blocked by #6
8. UI Purchase/Team/AI Credits (4c/4d/4e) — blocked by #4 dan #7
9. Newsletter & Feedback proxy (Fase 2) — blocked by #2
10. n8n/Resend + Tauri auto-updater (Fase 3) — blocked by #1
11. In-app feedback + crash reporting (Fase 5) — blocked by #9
12. Docs site (Fase 6) — blocked by #8

---

## Task 1 — Android Keystore Setup (BLOCKING)

```
You are setting up permanent Android app signing for CAFramework, an open-source Tauri v2 desktop/
mobile SSH client. Repo: /home/cecepazhar/Project/caframework, Android build config in
Dockerfile.android and .github/workflows/.

Context: current APK builds (dist/caframework-v2.1.5.apk etc.) have NO tracked permanent keystore —
each build may have been signed ad-hoc. Before the APK is opened to the public (labeled "Beta"
on the landing page), this must be fixed: if the signing key changes between releases, Android
refuses in-place updates and every installed user would need to uninstall/reinstall manually.

Do this:
1. Audit how current APK builds are signed — check Dockerfile.android, any keystore.jks or
   .p12 references, CI workflow steps, gradle/tauri android config. Report exactly what you
   find (file paths, whether a keystore already exists and is reused, or whether signing is
   ephemeral/debug-key per build). Do NOT assume — show the actual commands/config you checked.
2. If no permanent keystore exists: generate ONE new release keystore (keytool -genkeypair,
   RSA 2048+, validity 25+ years). Do NOT commit it to git.
3. Propose a secure storage plan for the keystore + its passwords (e.g. GitHub Actions secrets
   for CI signing, plus an offline backup copy) — ask me to confirm the storage location before
   finalizing, since losing this keystore permanently breaks future updates too.
4. Wire the CI workflow to use this keystore consistently for every future release build.
5. Document the keystore fingerprint (SHA-256) in a new file `docs/android-signing.md` so it's
   traceable in future.

Report your findings and what you did in Bahasa Indonesia, with concrete evidence (commands run,
file diffs, keystore fingerprint) — not a summary claiming it's "done" without showing proof.
If anything about current signing state is ambiguous, ask me before generating a new keystore
(overwriting/ignoring an existing one that's already in use would break existing installs).
```

---

## Task 2 — Privacy Policy (parallel dengan Fase 0)

```
You are drafting a Privacy Policy page for CAFramework's landing site, gcc/apps/caframework (Go + templ +
Tailwind, domain caframework.fathforce.com). This must go live BEFORE any email-capture form is
enabled on the site (a dependent task).

Context: CAFramework's core desktop app claims "zero telemetry, zero tracking" (true today per code
audit). The landing site will soon collect: (a) newsletter emails via a subscribe form, (b)
optional in-app feedback/testimonials with a name + free text, (c) payment/billing data once
Lemon Squeezy checkout goes live (handled by Lemon Squeezy as Merchant of Record, not stored by
Fathforce directly). Read caframework-pricing-tiers-v1.md and caframework-pro-licensing-architecture-v1.md
in the Notes folder for what data actually gets stored where (Turso: licenses/devices; Postgres:
newsletter/testimonials).

Do this:
1. Write a plain-language Privacy Policy covering: what's collected (email for newsletter,
   optional feedback text, license/device metadata for Pro users, payment via Lemon Squeezy),
   what's explicitly NOT collected (no telemetry from the desktop app, no vault contents ever
   leave the device), how to unsubscribe/delete data, contact for privacy questions.
2. Add it as a new templ page + route in gcc/apps/caframework, linked from the footer.
3. Keep it short — this is an indie open-source project, not an enterprise SaaS; don't
   over-lawyer it, but do cover GDPR/UU PDP basics (right to deletion, data location).

Report in Bahasa Indonesia what you wrote and where it's linked from. If you're unsure about a
factual claim (e.g. exactly how long email data is retained), ask me rather than guessing.
```

---

## Task 4 — Team Vault Key-Wrapping Design (DESIGN ONLY, read-only research)

```
You are doing a cryptographic protocol DESIGN task for CAFramework — this is a research/design
session, you must NOT write implementation code. Output is a design document only.

Context: CAFramework's vault is encrypted per-device from a DEK derived via Argon2id from the user's
local master password (see crates/caf-core/src/vault.rs and secret.rs in
/home/cecepazhar/Project/caframework). The Pro tier allows up to 7 people sharing one license. The
chosen design direction (confirmed 25 Sep 2026) is "Personal Vault + Team Vault": each user keeps
their own private vault as today, and can explicitly "publish" specific host entries to a
separate Team Vault that gets wrapped per-device (asymmetric key-wrapping), rather than sharing
one encryption key across everyone's full vault.

Do this:
1. Read crates/caf-core/src/vault.rs, secret.rs, and db.rs to understand the exact current
   encryption scheme (algorithms, key derivation, storage format) — cite line numbers for every
   claim you make about how it currently works.
2. Read caframework-pro-licensing-architecture-v1.md and caframework-pricing-tiers-v1.md §2 in the Notes
   folder for the device/license model this needs to integrate with.
3. Produce a design document answering: how is the Team Vault's key generated and rotated; how
   is it wrapped per-device (what public key does each device present, where is the device
   keypair generated/stored); what happens when a device is deactivated (does the team key need
   rewrapping for remaining devices, or can old devices retain stale access); what's the offline
   behavior (a device with no team vault access yet, syncing while offline); what's the minimal
   new Turso schema needed.
4. Explicitly list open questions / tradeoffs you are NOT confident about, rather than picking
   silently and moving on.

Save the output as caframework-team-vault-design-v1.md in the same Notes folder. Report your findings
in Bahasa Indonesia at the end, summarizing the recommended approach and open questions. This is
read-only/design-only — do not modify any code in caframework or gcc repos for this task.
```

---

## Task 5 — Verify Ollama Infra Capacity (research, read-only)

```
You are assessing whether Fathforce's existing self-hosted AI infrastructure (9Router/OpenClaw/
Hermes — see project memory / internal docs for where these run) has spare capacity to serve
CAFramework's planned "AI Ops Copilot hosted" feature: up to 300 AI requests/month pooled per Pro
license, shared across up to 7 people.

Do this:
1. Identify where the current Ollama/self-hosted model infra actually runs (which machine(s),
   what models are already loaded, current utilization if you have access to check).
2. Estimate expected load: assume some number of active Pro licenses (ask me for a realistic
   number if you don't have one — don't guess a business projection) × up to 300 req/month each,
   estimate concurrent request patterns (bursty during work hours vs spread out).
3. Report whether current infra can absorb this without degrading existing AI Team workloads
   (Hermes/OpenClaw), or whether it needs dedicated capacity/a queue/rate-limiting.
4. Recommend a model to use for this feature specifically — note that Prompt Studio's existing
   presets (Laravel, Docker, Node.js, UFW command generation, see caf-core/src/ai.rs for the
   current BYO-key pattern) need decent instruction-following; flag if a general-purpose
   self-hosted model is likely to underperform Claude/GPT for this specific task type.

This is read-only assessment — do not change any infra config. Report findings in Bahasa
Indonesia with concrete numbers/evidence where you have them, and explicit "I don't know, need
to check X" where you don't.
```

---

## Task 10 (extension) — Tauri Auto-Updater

```
You are adding self-update capability to the CAFramework Tauri v2 desktop app, to pair with the
existing release-notification email flow (see landing-parity-roadmap-v1.md §4 Fase 3 in the
Notes folder for the email side, already partly designed).

Repo: /home/cecepazhar/Project/caframework, Tauri app in crates/caf-app.

Do this:
1. Add the official Tauri v2 updater plugin (tauri-plugin-updater) to crates/caf-app.
2. Set up the update manifest/endpoint — decide with me whether this is hosted via GitHub
   Releases' standard latest.json convention or a custom endpoint on apps/caframework; ask me to
   confirm before choosing, since it affects the release CI workflow.
3. Wire signing for update artifacts (Tauri updater requires a separate minisign/signify keypair
   for update payload verification — NOT the same as the Android keystore from a separate task).
   Generate this keypair, do not commit the private key, document the public key location in
   tauri.conf.json.
4. Add the CI step (.github/workflows/release.yml) that generates and publishes the update
   manifest alongside each tagged release.
5. Add a simple in-app "Update available" notification UI (Settings or a small banner) that
   triggers the update flow — user-initiated, not silent/forced, to stay consistent with the
   app's "we don't do anything without you asking" trust posture.

Verify with `cargo check --workspace` and a real test release cycle if possible (even a draft
pre-release) before claiming this works. Report in Bahasa Indonesia what you built and how you
verified it, with actual command output as evidence.
```

---

## Task 11 (extension) — Crash Reporting Opt-In

```
You are adding OPT-IN anonymous crash reporting to the CAFramework Tauri v2 desktop app. This must
NOT contradict the app's "zero telemetry" claim — it must only ever trigger on an explicit user
action after a crash, never silently.

Repo: /home/cecepazhar/Project/caframework.

Do this:
1. Propose a crash reporting approach — recommend evaluating a self-hosted option (e.g.
   self-hosted Sentry, or GlitchTip as a lighter self-hosted alternative) over a third-party SaaS,
   given the project's privacy-first positioning; ask me to confirm the choice before
   integrating, since this has real infra cost implications.
2. Ensure NO automatic/background reporting — implement it so a crash produces a local prompt
   ("An error occurred. Send an anonymous report to help fix it?") that requires explicit user
   click to send. Nothing is transmitted without that click.
3. Scrub PII before anything is sent — no file paths containing the username, no vault contents,
   no SSH host/credentials ever included in a crash payload. Be explicit in your report about
   exactly what fields get scrubbed and how you verified nothing sensitive leaks (show an actual
   example payload).
4. Add a one-line mention of this in the app's existing privacy/telemetry documentation so it's
   consistent with the "zero telemetry unless you explicitly opt in" story.

Report in Bahasa Indonesia, with a real example of a scrubbed crash payload as evidence that PII
is actually removed, not just a claim that it is.
```


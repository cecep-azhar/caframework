# CAFramework — Autonomous Execution Prompt (`prompt-fix.md` v1.1)

> Give this whole file to the development agent together with `TASK_FIX.md` (v0.1.0), `TASK_FEATURES.md` (v0.2.0), `prompt-dev.md`, `prd.md` v1.1, `task.md` and the audit `2026-10-02-audit-statis-caframework.md`.
> v1.1 (4 Oct 2026): runs two plans in sequence and enforces the quality bars (stable, maintainable, scalable).
> Instructions are in English. **Every report, status update and question to the owner MUST be in Bahasa Indonesia.**
> This file **supplements** `prompt-dev.md`. Everything in `prompt-dev.md` still applies (stack §3, visual contract §4, architecture §5, vertical slice §6, pitfalls §7, report format §9) except where §1.3 below overrides it.

---

## 0. Configuration (owner fills in before starting)

```yaml
REPO_DIR: ~/Project/caframework          # absolute path to the framework repo
BASE_COMMIT: 7ec9e64                     # audited commit; F0 records drift if HEAD differs
WORK_BRANCH: fix/remediation-v2          # plan 1 (TASK_FIX.md, stages F0–F20)
FEATURE_BRANCH: feat/v0.2.0              # plan 2 (TASK_FEATURES.md, stages G0–G10), branched from the F20 gate commit
PLANS: [TASK_FIX.md, TASK_FEATURES.md]   # executed in this order
STOP_AFTER_PLAN: ""                      # "TASK_FIX.md" to stop after v0.1.0 work; empty = run both
RUNNER: any                              # "any" or "x1-bench" (owner's ThinkPad X1 Yoga Gen 3)
HAS_TAURI_DEPS: false                    # true if webkit2gtk/gtk dev packages are installed (VS-A runs locally)
CAN_PUSH: true                           # may push WORK_BRANCH to REMOTE (never main, never force)
REMOTE: origin
OWNER_CHANNEL: telegram                  # where stage reports go; "ledger-only" if no channel
CACASH_APP_TOML: ""                      # path to CACash app.toml if available (needed in F20.7)
MAX_TASK_ATTEMPTS: 3                     # attempts per task before STOP
SCRATCH_DIR: ""                          # temp dir for clones/dry runs; default: $(mktemp -d)
```

If a value is missing or contradicts the environment (for example `RUNNER: x1-bench` but no display), report it in Bahasa Indonesia and use the safest interpretation: `CAN_PUSH: false`, `HAS_TAURI_DEPS: false`.

## 1. Mission and precedence

### 1.1 Mission

Execute the plans in `PLANS` in order: `TASK_FIX.md` stages F0 → F20 on `WORK_BRANCH` (audit remediation + v1.1 foundations → v0.1.0), then `TASK_FEATURES.md` stages G0 → G10 on `FEATURE_BRANCH` (v1.1 features → v0.2.0), until each release's Definition of Done (PRD §9) is met with evidence or blocked only by owner actions. You prove every change with real command output, keep the framework product-agnostic, stable, maintainable and scalable (PRD §6, `TASK_FIX.md` §1.10), and never report work as done that is not wired end to end.

### 1.2 Sources of truth

| Question | Source |
|---|---|
| What the product must do | `prd.md` |
| Decisions already made | `TASK_FIX.md` §3 (D-1 … D-13), `TASK_FEATURES.md` §1 (E-1 … E-10), PRD §7 |
| What to change, in which order, and how to prove it | `TASK_FIX.md` §4, then `TASK_FEATURES.md` |
| Quality bars | `TASK_FIX.md` §1.10, PRD §6 |
| How to build features, style, stack, pitfalls | `prompt-dev.md` |
| How you operate during this remediation | this file |
| Current state of the work | `Notes/fix-ledger.md` (always wins over your memory) |

If two sources conflict and §1.3 does not resolve it, STOP (§3) and ask.

### 1.3 Overrides of `prompt-dev.md`

1. **Gates (`prompt-dev.md` §2.4, §8.2).** You do **not** stop for approval at each gate (D-8). When a gate passes, write the report, then continue to the next stage in the same session. You still never start a stage whose gate failed.
2. **Evidence location.** Evidence goes to `Notes/evidence/fix/F<n>.md` for plan 1 and `Notes/evidence/feat/G<n>.md` for plan 2 (not `S<n>.md`). `Notes/task.md` is ticked only in F20.5.
3. **Questions.** Questions that a §3 decision already answers are not asked again. New questions follow `prompt-dev.md` §2.1 (numbered, Bahasa Indonesia, recommended default), but you continue with other unblocked tasks while waiting unless the question is BLOCKING (§3).

## 2. Authority tiers

| Tier | You may | Examples |
|---|---|---|
| 0 — do it | without asking | edit tracked files in `REPO_DIR` on `WORK_BRANCH` / `FEATURE_BRANCH`; run builds, tests, linters, guards; create files under `Notes/`, `guards/`, `docs/`; create and delete temp dirs you created under `SCRATCH_DIR`; install dev tools in user space |
| 1 — do it and report | then list it in the stage report | add or remove a dependency (§1.9 of `TASK_FIX.md`); `git rm` tracked files named in `TASK_FIX.md`; push `WORK_BRANCH` / `FEATURE_BRANCH` when `CAN_PUSH: true`; create `FEATURE_BRANCH` at G0.1; open a draft PR to show CI |
| 2 — never | prepare commands, write an OWNER-VERIFY entry, continue elsewhere | merge to `main`; create tags; rewrite history or force-push; delete untracked files or anything outside `REPO_DIR`; touch the owner's real data dir (`~/.local/share/caframework`) or real keystores; create, rotate or upload real secrets; publish releases; change a §3 decision; weaken a test, guard, lint or requirement |

## 3. STOP conditions

Stop work on the affected task, write a blocker report (§6.5) and send it. Continue with other tasks or stages only if they do not depend on the blocked one; otherwise end the session with the final report (§7).

1. A gate fails after `MAX_TASK_ATTEMPTS` attempts on its failing task.
2. A fix would require a Tier 2 action.
3. Two sources conflict (§1.2) or the PRD does not cover a product, security, licensing or scope decision.
4. A finding turns out wrong or the code no longer matches the audit in a way that changes the plan (not a moved line, a different design).
5. You discover a real secret, credential or personal data in the repo or its history.
6. A dependency you need fails `cargo deny` / `npm audit` and no compliant alternative exists.
7. Free disk < 3 GB, or the runner lacks something the stage strictly needs (for example x1-bench-only work on `any`): record OWNER-VERIFY and continue with work that does not need it.
8. The owner sends "STOP" or "BERHENTI" on `OWNER_CHANNEL`.
9. A quality bar (`TASK_FIX.md` §1.10) cannot be met without weakening it.

Never "stop" by quietly reducing scope. Partial results are reported as SEBAGIAN with what is missing.

## 4. Run loop (one long session, resumable)

### 4.1 Session start (also after any interruption or context compaction)

1. `cd $REPO_DIR`; `git status --porcelain`; `git branch --show-current`. If the tree is dirty from an earlier session, inspect the diff: finish and commit it if it belongs to the current task, otherwise stash it with the message `wip-recovered-<date>` and record that in the ledger.
2. Read `Notes/fix-ledger.md` (if it does not exist, you are at F0). Identify: current plan and stage, last completed task, open OWNER-VERIFY entries, open questions, owner answers received since the last session.
3. Read the **current stage section** of the current plan in full, plus `TASK_FIX.md` §1 (rules and quality bars) and the decision tables. Do not work from memory of earlier stages.
4. Post a short start message (Bahasa Indonesia) on `OWNER_CHANNEL`: stage, next task, anything waiting for the owner.
5. If the branch in the working tree is not the current plan's branch, check out the right one (both exist only after G0.1).

### 4.2 Per stage

1. **Plan.** In the ledger, set the stage to `IN_PROGRESS`; list its tasks; note anchors to re-locate (`TASK_FIX.md` §1.6).
2. **Per task:**
   1. Re-locate anchors with `git grep -n`; record drift in the evidence file.
   2. Write the failing test first when the task changes behaviour.
   3. Implement the smallest correct change (`prompt-dev.md` §2.5).
   4. Run the task's **Verify** commands; save raw output to `Notes/evidence/fix/logs/F<n>-<slug>.log` and paste the relevant part into `F<n>.md` under the task.
   5. Check every **Accept** item against the output. Mark the task `DONE` in the ledger only if all are proven; otherwise `PARTIAL`/`UI-ONLY`/`NOT_VERIFIED`/`BLOCKED` with the reason.
   6. Commit (§5.3). If `CAN_PUSH`, push `WORK_BRANCH`.
3. **Gate.** Run VS-R, VS-F, VS-G (and VS-A if `HAS_TAURI_DEPS`) from a clean state (`cargo clean -p caf-core` is not required; a fresh `npm ci` is). Record results in `F<n>.md`. Gate passes only if every task is `DONE` or explicitly allowed as OWNER-VERIFY by the stage text.
4. **Handoff note.** In the ledger, write a 5–10 line handoff for the stage: what changed, new files, new commands, decisions taken, what the next stage must know. This is what you rely on after context loss.
5. **Report.** Send the stage report (§6.3) in Bahasa Indonesia.
6. **Continue** to the next stage (D-8), starting again at §4.1 step 3 (re-read the next stage section).
7. **Plan switch.** When F20 reaches `GATE_PASSED (agent)` (owner actions such as tagging may still be pending) and `STOP_AFTER_PLAN` is not `TASK_FIX.md`, send the v0.1.0 final report (§7), then start `TASK_FEATURES.md` at G0. Owner actions pending for v0.1.0 stay in the OWNER-VERIFY queue.

### 4.3 Context hygiene

- Keep long command output in log files, not in your working context; paste only what proves the Accept items.
- After every stage, and whenever you notice you are unsure what was done, re-read the ledger instead of guessing.
- Never re-do a `DONE` task unless a later change broke it; if it did, reopen it in the ledger with the reason.

## 5. Working rules

### 5.1 Proof

- `prompt-dev.md` §2.2 and §2.3 apply to every line you write: no "done", "works", "safe", "fixed" without the command and its real output. If you could not run something, write **BELUM DIVERIFIKASI** and why.
- A guard or test counts only after you showed it failing on a bad fixture (`TASK_FIX.md` §1.2.6).
- Wiring proof for user-facing features follows `TASK_FIX.md` §1.2.4. A page over a stub is **UI SAJA** in reports.
- Do not edit, delete, skip or `#[ignore]` an existing test to make the suite pass. If a test is wrong, explain why in the evidence and fix it to assert the correct behaviour in the same commit.

### 5.2 Safety

- Every run of the app, CLI or smoke script uses `export CAF_DATA_DIR="$(mktemp -d)"`. Never read or write the owner's real data dir.
- No network access except package registries, GitHub and local mock servers. All HTTP behaviour is tested against mock servers on `127.0.0.1`.
- No secrets in code, logs, evidence, commit messages or test fixtures. Test passwords and recovery words are generated per run; evidence shows scan summaries, never the secret values.
- Generated outputs (`new-app` runs, dry-run clones) go under `SCRATCH_DIR`, never inside `REPO_DIR`.

### 5.3 Commits

- Small, one concern each, English messages: `fix(F<n>.<k>): <what>` / `feat(G<n>.<k>): <what>` / `test(F<n>.<k>): …` / `chore(F<n>.<k>): …` / `docs(F<n>.<k>): …`.
- Generated files in their own commit: `chore(codegen): regenerate from app.toml`.
- A commit that claims verification must include the evidence file change that proves it.
- Only commit on the current plan's branch. Never amend pushed commits.

### 5.4 Dependencies

Follow `TASK_FIX.md` §1.9. List each in the stage report: name, version, licence, reason, `cargo deny`/`npm audit` result.

### 5.5 Quality bars and genericity

- Every new line of code meets `TASK_FIX.md` §1.10: no panicking calls in core, paginated lists, atomic writes, tests and docs for public items, file/function size limits, benchmarks for data-heavy paths.
- Extend through registries (menus, settings, permissions, entities, integrations, jobs, AI context providers); never hard-code a product term or a role name in framework code (`guard --only genericity`, `guard --only role_literals`).
- Brand data comes only from generated brand files; never hard-code the owner's contact details.

### 5.6 When the audit is wrong

If a prediction is REFUTED in F0 or a finding no longer applies, keep the task, prove the actual state with output, and mark the task `DONE (sudah sesuai)` only when its Accept items are proven by that output.

## 6. Templates

### 6.1 `Notes/fix-ledger.md`

```markdown
# Fix ledger — CAFramework remediation
Base commit: <hash> · Branches: fix/remediation-v2, feat/v0.2.0 · Started: <date> · Runner: <label>
Current plan: TASK_FIX.md | TASK_FEATURES.md

## Status
| Stage | Status | Gate | Evidence | Handoff |
|---|---|---|---|---|
| F0 | TODO / IN_PROGRESS / GATE_PASSED (agent) / DONE / BLOCKED | <date> | Notes/evidence/fix/F0.md | see below |
| … G10 | | | Notes/evidence/feat/G10.md | |

## Tasks
| Task | Status | Evidence anchor | Note |
|---|---|---|---|
| F0.1 | TODO / DONE / PARTIAL / UI-ONLY / NOT_VERIFIED / BLOCKED | F0.md#f01 | |

## Owner decisions (copied from TASK_FIX.md §3 and TASK_FEATURES.md §1)
| ID | Decision | Date | Changed? |

## OWNER-VERIFY queue
| ID | Stage/task | What | Blocking? | Status | Owner output |

## Questions to owner
| # | Question (Bahasa Indonesia) | Recommendation | Blocking? | Answer | Date |

## Handoff notes
### F0
<5–10 lines>
```

### 6.2 `Notes/evidence/fix/F<n>.md`

```markdown
# Evidence <F|G><n> — <stage title>
Commit range: <first>..<last> · Runner: <label> · Date: <date>

## Anchors re-located
| Audit anchor | Now at | Note |

## F<n>.<k> <task title>
Command: `<exact command>`
Output:
<real output, trimmed but unedited; full log: logs/F<n>-<slug>.log (<bytes>, sha256 <hash>)>
Accept:
- [x] <criterion> — proven by <line/output above>
- [ ] <criterion> — NOT VERIFIED: <reason> → OWNER-VERIFY <id>
Verdict: VERIFIED | PARTIAL | UI-ONLY | NOT VERIFIED (<reason>)

## Gate
| Set | Command | Exit | Log |
| VS-R | … | 0 | logs/F<n>-vs-r.log |
```

### 6.3 Stage report (Bahasa Indonesia; extends `prompt-dev.md` §9)

```
LAPORAN TAHAP <F|G><n> — <nama tahap>
Status: SELESAI TERVERIFIKASI | GATE LULUS (MENUNGGU VERIFIKASI PEMILIK) | SEBAGIAN | TERBLOKIR
Ringkasan (maks. 5 baris):
Tugas:
  - F<n>.<k> <judul> — TERVERIFIKASI / UI SAJA / BELUM DIVERIFIKASI / TERBLOKIR
    Bukti: `<perintah>` -> <output asli, dipotong seperlunya>
Gate: VS-R <ok/gagal> · VS-F <…> · VS-G <…> · VS-A <…/belum: OWNER-VERIFY-xx>
Ratchet: <nama>: <sebelum> -> <sesudah>
Kualitas: coverage core <x>% / frontend <y>% · benchmark vs baseline <±z%> · guard ukuran/genericity <ok/gagal>
Temuan audit yang ditutup: <ID, …>
Yang TIDAK bisa saya verifikasi (dan perintah untuk dijalankan pemilik):
Dependensi baru (nama, versi, lisensi, alasan) / tidak ada:
Risiko / temuan baru:
Pertanyaan untuk pemilik (bernomor, dengan rekomendasi default):
Langkah berikutnya: <F|G><n+1> — <judul> (lanjut otomatis)
```

### 6.4 OWNER-VERIFY entry

```
OWNER-VERIFY-<nn> (F<n>.<k>) — <judul singkat>
Blocking: ya/tidak (jika ya: memblokir <tahap/aksi>)
Di mana: laptop X1 Yoga / perangkat Android / GitHub
Langkah (salin-tempel):
  export CAF_DATA_DIR="$(mktemp -d)"
  <perintah 1>
  <perintah 2>
Hasil yang diharapkan: <apa yang harus terlihat>
Tempel output / tangkapan layar di: Notes/evidence/fix/F<n>.md#owner-verify-<nn>
```

### 6.5 Blocker report

```
TERBLOKIR — F<n>.<k> <judul>
Penyebab: <satu kalimat>
Bukti: `<perintah>` -> <output>
Yang sudah dicoba (maks. 3): …
Pilihan:
  1. <opsi> — dampak: … (rekomendasi)
  2. <opsi> — dampak: …
Sementara menunggu, saya lanjut ke: <tugas/tahap yang tidak bergantung> | tidak ada (sesi berhenti)
```

## 7. Final report

At the end of each plan (F20 for v0.1.0, G10 for v0.2.0), or when nothing unblocked remains, send in Bahasa Indonesia:

```
LAPORAN AKHIR <REMEDIASI v0.1.0 | FITUR v0.2.0> CAFRAMEWORK
Status DoD <v0.1.0 | v0.2.0>: TERPENUHI | BELUM (sisa: …)
Tahap: <n> DONE · <n> GATE LULUS (menunggu pemilik) · <n> TERBLOKIR
Butir audit: <n>/73 tertutup dengan bukti · Temuan: <n>/25 tertutup (v0.1.0) | Fitur v1.1: <n>/<m> TERVERIFIKASI (v0.2.0)
Kualitas: coverage core/frontend · anggaran performa terpenuhi <n>/<m> · soak test <ok/gagal>
Hasil re-audit read-only: kritis <n> · tinggi <n> · UI saja <n>
Antrian pemilik (urut prioritas): OWNER-VERIFY-xx …
Aksi Tier 2 yang disiapkan (perintah ada di Notes/publication-plan.md, F20.md, G10.md): merge, tag, purge history, pindah Notes/ ke repo privat
Cold start: median <ms> / p90 <ms> (atau belum diukur)
```

---

## Appendix A — Start message (owner sends this to the agent)

```
Read prompt-fix.md, prompt-dev.md, TASK_FIX.md, TASK_FEATURES.md, prd.md (v1.1), task.md and the audit
2026-10-02-audit-statis-caframework.md in full. Configuration (prompt-fix.md §0):
<paste filled YAML>

Start with prompt-fix.md §4.1. If Notes/fix-ledger.md exists, resume from it; otherwise begin at F0.
Work autonomously through F0–F20, then G0–G10 (decision D-8). Stop only for the STOP conditions in §3.
All reports to me in Bahasa Indonesia.
```

To resume after an interruption, send:

```
Resume CAFramework remediation. Follow prompt-fix.md §4.1 exactly: read Notes/fix-ledger.md,
re-read the current stage section of the current plan, then continue. Owner updates since last session:
<answers / OWNER-VERIFY outputs, or "none">
```

## Appendix B — Read-only QA re-audit prompt (F20.4 and G10.2, run in a separate session)

```
You are a QA auditor for CAFramework. READ-ONLY: do not create, edit, or delete any file outside
Notes/audit/. You may read files and run tests, linters, builds and the app with a temporary data dir
(export CAF_DATA_DIR="$(mktemp -d)"). Write findings only to Notes/audit/<YYYY-MM-DD>-reaudit.md.

References: prd.md (v1.1), task.md, prompt-dev.md, TASK_FIX.md, TASK_FEATURES.md, Notes/audit/2026-10-02-audit-statis-caframework.md,
Notes/fix-ledger.md, Notes/evidence/fix/.

1. Re-score all 73 items of the 2 October audit (same IDs 1.1–7.7) as PASS / PARTIAL / FAIL / NOT VERIFIED,
   each with command + output or file:line. Do not trust the evidence files: re-run the key commands yourself.
2. Re-check all 25 findings (C-1, T-1..T-10, S-1..S-9, R-1..R-5): CLOSED / OPEN / REGRESSED, with proof.
3. Run: VS-R, VS-F, VS-G, VS-A (if Tauri deps exist), VS-E as defined in TASK_FIX.md §1.1.
4. Security probes:
   - call every OwnerOnly command as a non-owner and without a session (expect CAF-AUTH-*);
   - try to read another profile's private rows through every data command (expect none, including the owner role);
   - confirm no vault.key is ever created; the DB file does not start with "SQLite format 3";
   - scan the data dir for the password, recovery words, DEK hex after setup;
   - gitleaks on the full history; check localStorage use; check AI payloads at each privacy level against a mock server.
5. Wiring: for every module and the Notes slice trace UI -> binding -> command -> core -> DB -> restart read-back.
   Classify WIRED / PARTIAL / UI-ONLY with evidence.
6. Generator: run the E2E; inspect outputs for leftover caframework strings, shared updater keys, missing .github.
7. UI: both themes, desktop and 390 px width, four states per data page, i18n parity, 44 px touch targets.
8. v1.1 features (re-audit for v0.2.0): RBAC matrix enforced in core for every command and default role; only the super role
   reads others' private rows and each read is audited; super role invariants (cannot lose "*", last super-role profile);
   integration secrets never reach the frontend or logs; outbox idempotency; scheduler survives failing/panicking jobs and
   catches up; import validation, transactions, visibility and formula-injection protection; Pro gating enforced in core,
   tampered licence rejected, dev unlock absent in release builds; [brand] shown in About/lock screen/installer metadata;
   wizard and App Builder produce identical configs; no product-specific terms in framework code.
9. Quality bars (PRD §6): clippy strict set, coverage thresholds, file/function size guard, pagination arch test,
   benchmark budgets on the reference dataset, soak test, fuzz targets. Report each bar as MET / NOT MET with output.

Report in Bahasa Indonesia. Each finding: severity (kritis/tinggi/sedang/rendah), location, evidence
(command/output or file:line), suggested fix. Never state something is fine without the proof that shows it.
End with: counts per severity, number of UI-ONLY items, and whether PRD §9 is met.
```

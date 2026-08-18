# Dead-Code Cleanup Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove provably unreachable Rust code, then remove only demonstrably obsolete public API while preserving supported workspace behavior.

**Architecture:** Use compiler dep-info plus Graft's exhaustive references for the conservative pass, followed by a separate compatibility review for exported items. Structural checks provide the red/green cycle for deletions; crate and workspace verification protect runtime behavior.

**Tech Stack:** Rust 2024, Cargo workspace tooling, Graft call graph, Git/GitHub CLI.

---

## File structure

- Delete seven orphaned Rust files that are absent from every all-target/all-feature compiler dep-info graph.
- Modify FMI module and trait files to remove unconditional `#[cfg(false)]` blocks.
- Retain the published `UnknownsTuple` alias: it is unused internally but externally usable and has no named replacement, so record it as a deferred breaking-change candidate.
- Modify the FMI export trait only to remove the redundant `#[allow(dead_code)]` from `TerminalProvider`.
- Modify FMI simulation and test-data tests to remove disabled diagnostic/test helpers.
- Create branch-local `graft/` review context after the source tree changes; it is untracked, must not be committed, and is regenerable.

### Task 1: Remove orphaned source files

**Files:**
- Delete: `fmi-export-derive/src/codegen/util.rs`
- Delete: `fmi-export-derive/src/codegen/value_ref.rs`
- Delete: `fmi-export-derive/src/tests.rs`
- Delete: `fmi-sim/src/main.old.rs`
- Delete: `fmi/src/fmi2/variable.rs`
- Delete: `fmi/src/fmi3/model.rs`
- Delete: `fmi/src/fmi3/model2.rs`
- Modify: `fmi-export-derive/src/lib.rs:12-14`
- Modify: `fmi/src/fmi3/mod.rs:3-8`
- Modify: `fmi/src/fmi3/import.rs:81-92`

- [ ] **Step 1: Run the structural test and verify RED**

```bash
for path in \
  fmi-export-derive/src/codegen/util.rs \
  fmi-export-derive/src/codegen/value_ref.rs \
  fmi-export-derive/src/tests.rs \
  fmi-sim/src/main.old.rs \
  fmi/src/fmi2/variable.rs \
  fmi/src/fmi3/model.rs \
  fmi/src/fmi3/model2.rs
do
  test ! -e "$path" || exit 1
done
```

Expected: exit status 1 because the orphaned files still exist.

- [ ] **Step 2: Delete the orphaned files and their disabled declarations**

Delete all seven files with `apply_patch`. Also remove:

```rust
// fmi-export-derive/src/lib.rs
//#[cfg(test)]
//mod tests;

// fmi/src/fmi3/mod.rs
#[cfg(false)]
pub mod model;

// fmi/src/fmi3/import.rs
/// Build a derived model description from the raw-schema model description
#[cfg(false)]
pub fn model(&self) -> &model::ModelDescription {
    &self.model
}
```

- [ ] **Step 3: Verify GREEN with structural and compiler-graph checks**

Run the Step 1 loop again; expect exit status 0. Then run:

```bash
cargo check --workspace --all-targets --all-features
comm -23 \
  <(git ls-files '*.rs' | sort) \
  <(find target -name '*.d' -type f -exec awk '{ for (i = 1; i <= NF; i++) if ($i ~ /\.rs$/) print $i }' {} + \
    | sed -e 's#^\./##' -e "s#^$(pwd)/##" \
    | sort -u)
```

Expected: Cargo succeeds and the comparison prints no tracked Rust source files.

- [ ] **Step 4: Commit the conservative orphan cleanup**

```bash
git add -u
git commit -m "refactor: remove orphaned Rust sources"
```

### Task 2: Remove unconditionally disabled blocks

**Files:**
- Modify: `fmi-schema/src/fmi2/model_description.rs:112-240`
- Modify: `fmi/src/fmi2/instance/common.rs:238-281`
- Modify: `fmi/src/fmi2/instance/traits.rs:167-168`
- Modify: `fmi/src/fmi3/instance/common.rs:421-427`
- Modify: `fmi/src/fmi3/traits.rs:3-6,100-109`
- Modify: `fmi-sim/tests/test_fmi_sim.rs:3-14,229-313`
- Modify: `fmi-test-data/src/lib.rs:283-301`

- [ ] **Step 1: Run the disabled-code test and verify RED**

```bash
if rg -n '#\[cfg\(false\)\]' --glob '*.rs' --glob '!target/**' --glob '!graft/**'; then
  exit 1
fi
```

Expected: exit status 1 with the remaining disabled blocks listed.

- [ ] **Step 2: Remove the disabled implementations and declarations**

Use `apply_patch` to delete every remaining `#[cfg(false)]` item:

- the incomplete FMI 2 schema lookup, unknown mapping, output, derivative, initial-unknown, index, and continuous-state methods;
- the Arrow-based FMI 2 `set_values` trait method and implementation;
- the incomplete FMI 3 FMU-state trait methods and implementation;
- the disabled FMI simulation bouncing-ball test;
- the two disabled reference-FMU printing tests.

Delete `compare_record_batches` and `compare_f64_column_by_name`, which have no callers after the disabled bouncing-ball test is removed. Remove imports that become unused, including `Float64Type` and `Error` where Cargo reports them.

- [ ] **Step 3: Verify GREEN for affected crates**

```bash
if rg -n '#\[cfg\(false\)\]' --glob '*.rs' --glob '!target/**' --glob '!graft/**'; then
  exit 1
fi
cargo check -p fmi-schema -p fmi -p fmi-sim -p fmi-test-data --all-targets --all-features
```

Expected: no disabled-code matches and Cargo succeeds without warnings.

- [ ] **Step 4: Commit the disabled-code cleanup**

```bash
git add fmi-schema/src/fmi2/model_description.rs \
  fmi/src/fmi2/instance/common.rs \
  fmi/src/fmi2/instance/traits.rs \
  fmi/src/fmi3/instance/common.rs \
  fmi/src/fmi3/traits.rs \
  fmi-sim/tests/test_fmi_sim.rs \
  fmi-test-data/src/lib.rs
git commit -m "refactor: remove unconditionally disabled code"
```

### Task 3: Review public API compatibility and remove the redundant allowance

**Files:**
- Review only: `fmi-schema/src/fmi2/mod.rs:29` (`UnknownsTuple`)
- Modify: `fmi-export/src/fmi3/traits/mod.rs:147-151` (`TerminalProvider`)

- [ ] **Step 1: Verify the published alias has no internal consumers**

```bash
rg -n 'UnknownsTuple' --glob '*.rs' --glob '!target/**' --glob '!graft/**'
graft grep "UnknownsTuple"
```

Expected: the live Rust search finds exactly the `UnknownsTuple` alias. This proves it is unused internally, but because it is published, externally usable, and has no named replacement, it must remain compatible API.

- [ ] **Step 2: Retain the alias and remove only the redundant allowance**

Do not edit `UnknownsTuple`. Record it as a deferred breaking-change candidate; any future removal requires an explicit compatibility decision and a named replacement or major-version plan. Use `apply_patch` only to remove the redundant `#[allow(dead_code)]` attribute from `TerminalProvider`; retain the trait because derive output and `test_terminals_default` consume it.

- [ ] **Step 3: Record retain decisions for ambiguous public candidates**

Confirm with Graft and retain:

- `UnknownsTuple`, because it is published, externally usable, and has no named replacement even though the live search finds no internal consumers;
- `EventFlags::reset`, because it is a valid state-reset operation for downstream users;
- `TerminalProvider`, because derive-generated implementations and tests consume it;
- `Instance::callbacks`, because the owned callback box keeps FMI 2 callback pointers alive;
- interpolation implementations, because FMI 2/FMI 3 input paths use `PreLookup` and the type-specific implementations support runtime Arrow types;
- examples-harness helpers, because they are integration-test infrastructure.

These decisions go into the pull-request description; `UnknownsTuple` is specifically a deferred breaking-change candidate. They do not produce source edits.

- [ ] **Step 4: Verify the compatibility decision and redundant-allowance cleanup**

```bash
rg -n 'UnknownsTuple' --glob '*.rs' --glob '!target/**' --glob '!graft/**'
cargo test -p fmi-schema --all-features
cargo test -p fmi-export --all-features --test test_terminals_default
cargo fmt --all -- --check
```

Expected: the search prints exactly the retained alias, both test commands pass, and formatting reports no changes.

- [ ] **Step 5: Commit the redundant-allowance cleanup**

```bash
git add fmi-export/src/fmi3/traits/mod.rs
git commit -m "refactor: remove redundant dead-code allowance"
```

### Task 4: Refresh repository context and verify the workspace

**Files:**
- Local review context only: untracked `graft/**` as produced by the deterministic graph refresh

- [ ] **Step 1: Format and create branch-local Graft review context**

```bash
cargo fmt --all
graft build .
```

Expected: formatting succeeds. `graft build .` creates local, untracked review context and refreshes affected spans; it must not be staged, committed, added to `.gitignore` or `.ignore`, or otherwise recorded in shared metadata.

- [ ] **Step 2: Run final verification**

```bash
cargo fmt --all -- --check
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
for package in fmi fmi-schema fmi-export-derive fmi-sim fmi-test-data; do
  cargo package -p "$package" --list >/dev/null
done
graft map
for path in \
  fmi-export-derive/src/codegen/util.rs \
  fmi-export-derive/src/codegen/value_ref.rs \
  fmi-export-derive/src/tests.rs \
  fmi-sim/src/main.old.rs \
  fmi/src/fmi2/variable.rs \
  fmi/src/fmi3/model.rs \
  fmi/src/fmi3/model2.rs
do
  test ! -e "$path" || exit 1
done
git diff --check main...HEAD
```

Expected: every command succeeds. `graft map` reports `149 files · 1690 symbols · 730 edges`, and the loop confirms all seven deleted paths remain absent. If the temporary-project integration test requires crates.io, rerun that specific test with approved network access and record both the environmental failure and successful rerun.

- [ ] **Step 3: Retain the local graph through review without committing it**

```bash
git status --short graft .gitignore .ignore
```

Expected: `graft/` remains local and untracked, with no `.gitignore`, `.ignore`, or shared-metadata change. Keep it available through final review; after review it can be regenerated with `graft build .`. No graph commit is expected.

### Task 5: Review and open the pull request

**Files:**
- Review only: all changes relative to `main`

- [ ] **Step 1: Review the complete diff and commit history**

```bash
git status --short
git log --oneline main..HEAD
git diff --stat main...HEAD
git diff main...HEAD
```

Expected: only the approved design/plan, dead-code removals, necessary import cleanup, and the redundant `TerminalProvider` allowance removal are present. Graft artifacts are local review context and must be excluded from the final diff and pull request.

- [ ] **Step 2: Perform review and completion workflows**

Invoke `superpowers:requesting-code-review`, resolve verified findings, then invoke `superpowers:verification-before-completion` and `superpowers:finishing-a-development-branch`.

- [ ] **Step 3: Push and open the PR**

```bash
git push -u origin chore/remove-dead-code
gh pr create --base main --head chore/remove-dead-code \
  --title "refactor: remove dead code" \
  --body-file /tmp/rust-fmi-dead-code-pr.md
```

The PR body must summarize conservative removals, the public-API decision, retained candidates, compatibility impact, and exact verification results.

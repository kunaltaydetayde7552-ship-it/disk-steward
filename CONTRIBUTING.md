# Contributing

Thanks for helping. The most valuable contributions are new targets for the catalog and real-world reports, because
every cache behaves differently on APFS.

## Development setup

Requirements: macOS (the app relies on APFS), Rust (stable), [Bun](https://bun.sh), the Tauri CLI and the engine.

```sh
cargo install tauri-cli --version "^2" --locked
cargo install auto-compress-apfs
cd ui && bun install && cd ..
cargo tauri dev
```

Use `PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1` with `bun install` if any dependency tries to download a browser.

## Running checks

Run the checks relevant to your change before opening a pull request.

```sh
cd ui && bun test                                   # UI unit tests
cd ui && bun run build                              # type-checks (vue-tsc) and builds the UI
cargo test -p steward-core                          # catalog, measurement, risk, engine wrapper
cargo test -p disk-steward                          # Tauri shell (src-tauri)
cargo clippy --workspace --all-targets -- -D warnings
```

The UI build must exist before the Tauri crate compiles (`ui/dist` is embedded by Tauri).

## Project rules

- **Tests never touch the real `HOME`.** Use a temporary `HOME` and fixtures. The code takes a `home` path for exactly
  this reason (`Engine { bin, config, home }`).
- **No `apply` against real data in tests.** Tests that exercise compression use fixtures and a temporary engine
  config, never your own caches.
- Clean up what a test creates: temporary directories, config files and background processes.
- Keep files under 500 lines. Split by responsibility instead of growing a file.
- Code, identifiers, file names, tests, comments and commit messages are in English. Only user-visible UI text is
  localized, in `ui/src/locales/` (`pt-BR.ts` and `en.ts`). Add every new string to both.
- Reuse before adding: search for an existing helper or pattern first.
- Keep changes focused. Do not refactor unrelated code in the same pull request.
- Do not add vendor binaries, large screenshots or secrets to the repository. `design/screenshots/` is git-ignored;
  images used by the docs go in `docs/images/`.

## Adding a target to the catalog

Targets live in `builtin()` in [`crates/steward-core/src/catalog.rs`](crates/steward-core/src/catalog.rs). Each one is an
`Entry` with an id, a path, a kind (`Toolchain`, `PackageCache`, `Store`, `Build`) and a rule.

1. **Assess the risk first.** Open a
   [target request](https://github.com/lucasbrito-wdt/disk-steward/issues/new?template=target_request.yml) or answer
   these in your pull request:
   - How does the tool put files there: copy, hardlink, symlink or `clonefile`?
   - Do other folders (projects, environments) share blocks with it? Run `steward measure <path>` and compare the
     on-disk and private sizes. Heavy sharing means compression would store the data twice.
   - Is the folder rewritten constantly (build output, logs, databases)? Compression is undone on every rewrite.
   - Is it safe to read through transparent compression (no tool that needs raw blocks or `mmap` of the files)?
2. **Pick the rule:**
   - `Rule::Allowed` when the contents are written once and read many times, and measurement can decide the rest. The
     private-size check in [`risk.rs`](crates/steward-core/src/risk.rs) still blocks a target with too many shared bytes.
   - `Rule::VenvCloneScan` is specific to the uv cache. Do not reuse it for something else without discussion.
   - `Rule::Blocked(risk, reason)` when the folder must never be compressed. The reason is a one-line English message
     shown in the UI.
3. **Add exclusions** through `excludes` when only part of the folder is unsafe (the uv cache does this for its
   ephemeral `uvx` environments).
4. **Add a test** next to the existing ones in `catalog.rs`, using a temporary `HOME`. If a new risk string is
   user-facing, check how the UI shows it and add translations to both locale files.
5. **Update the docs.** Add the row to the built-in catalog table in [docs/API.md](docs/API.md) and mention the target in
   the README if it is user-visible.

When in doubt, block the target and explain why. A blocked entry is cheap to relax later; a wrong "allowed" can cost
someone disk space.

## Reporting a bug

Open a [bug report](https://github.com/lucasbrito-wdt/disk-steward/issues/new?template=bug_report.yml) with the macOS
version, the Disk Steward version, the output of `auto-compress-apfs --version` and `steward targets --json`. Redact
home paths and anything private.

## Pull requests

1. Open an issue first for anything larger than a small fix, so the approach can be agreed.
2. Fork, then branch from `main` with a descriptive name.
3. Make the change with tests where logic is involved.
4. Run the checks above.
5. Open the pull request and fill in the template. Describe what changed, why, and how you tested it.

Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/): `feat(catalog): add pip cache`,
`fix(risk): handle empty private size`, `docs: clarify install`.

## License

This project is licensed under [GPL-3.0-only](LICENSE). By submitting a contribution you agree that it is licensed under
the same terms.

By participating you also agree to the [Code of Conduct](CODE_OF_CONDUCT.md).

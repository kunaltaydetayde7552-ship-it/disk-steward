## Summary

<!-- What does this change and why? Link the issue it closes, if any. -->

## Type

- [ ] Bug fix
- [ ] New feature
- [ ] New or changed catalog target
- [ ] Docs
- [ ] Other

## Testing

- [ ] `cargo test -p steward-core`
- [ ] `cargo test -p disk-steward`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `bun test` and `bun run build` in `ui/`

## Checklist

- [ ] Tests use a temporary `HOME` and fixtures, never real data, and never run `apply` on real folders
- [ ] Code and comments are in English; new UI strings are added to pt-BR and en
- [ ] Files stay under 500 lines
- [ ] For a new target: risk assessed (how files are installed, clone sharing, rewrite frequency) and `docs/API.md` updated
- [ ] I agree my contribution is licensed under GPL-3.0

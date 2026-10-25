# Contributing

Thank you for helping with Caveat. Contributions from people and from agents
are welcome; [AGENTS.md](AGENTS.md) has the agent-specific guidance.

## Reporting

- **Bugs and documentation gaps:** open an
  [issue](https://github.com/WSattazahn/caveat-lang/issues) with the version or
  revision, a small program, event history or command that shows the problem,
  what you expected and what happened. Search open and closed issues first.
- **Features:** use the
  [agent feature request](https://github.com/WSattazahn/caveat-lang/issues/new?template=agent-feature-request.md)
  template, or a plain issue, and say what real use needs it.
- **Vulnerabilities:** report privately as the [security policy](SECURITY.md)
  describes, never in a public issue.

## Changing the code

Open a pull request against `main`. A pull request is accepted when:

1. **It is tested.** New behavior comes with tests that fail without it: Rust
   tests in `runtime/tests/`, kit tests in `kit/test/`, or scenarios. A bug fix
   adds a regression case named after the finding it fixes. Changed documented
   behavior updates the documentation in the same pull request.
2. **CI is green.** The [runtime workflow](.github/workflows/runtime.yml) and
   format check run on every pull request.
3. **It keeps the 1.x promise.** A 1.x minor only adds: the same programs give
   the same output bytes and exit statuses ([COMPATIBILITY.md](docs/COMPATIBILITY.md)).
   A change to what the language means is its own specification pull request
   in `spec/`, with a sweep of the existing programs it affects.

## Checks to run locally

```sh
cargo fmt --manifest-path runtime/Cargo.toml --all -- --check
cargo clippy --manifest-path runtime/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path runtime/Cargo.toml
npm ci
npm run build
npm run test:kit
```

The Rust code follows `rustfmt` and has no Clippy warnings; the CI treats
warnings as errors. JavaScript follows the style of the file it is in.
[What is checked, and how](docs/VERIFICATION.md) describes the proofs,
conformance checks and test suites, and [the README](README.md#verify) lists
the game and browser suites.

## License

By contributing you agree that your contribution is licensed under the
[MIT License](LICENSE), like the rest of Caveat's code and documentation.

# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.1.x | ✅ Active development |

We recommend always using the latest version. Security fixes are backported to the latest minor version upon request.

## Reporting a Vulnerability

The motarjim project takes security seriously. If you discover a security vulnerability, **please do not open a public GitHub issue**.

### Private Reporting

Report vulnerabilities privately through one of these channels:

1. **GitHub Security Advisories** (preferred):
   - Go to [https://github.com/motarjim/motarjim/security/advisories/new](https://github.com/motarjim/motarjim/security/advisories/new)
   - Click "Report a vulnerability"

2. **Email** the maintainers directly (see repository profile for contact information)

### What to Include

- **Type of vulnerability** (e.g., arbitrary code execution, denial of service, data exposure)
- **Steps to reproduce** — Minimal HTML/CSS input that triggers the issue
- **Affected versions** — Versions confirmed to be vulnerable
- **Potential impact** — What an attacker could achieve
- **Suggested fix** (optional) — If you have a proposed patch

### Response Timeline

| Timeframe | Action |
|-----------|--------|
| 48 hours | Acknowledgment of receipt |
| 5 business days | Initial assessment and plan |
| 30 days | Fix released (or extension communicated) |

## Security Practices

### Local-First Architecture

motarjim is a **local-first compiler**. By design:

- **No data leaves your machine.** HTML/CSS input is processed entirely locally.
- **No telemetry.** The compiler does not phone home, track usage, or collect analytics.
- **No network access required.** All compilation happens in-process (except optional AI enhancement).
- **No persistent storage of user content.** Generated output is written only where the user specifies.

### AI Enhancement Safety

When using `--ai-enhance` (or the equivalent API option):

- The AI model (Ollama) runs **locally on your machine**.
- HTML/CSS content **never leaves your computer**.
- The Ollama API endpoint defaults to `http://localhost:11434`.
- No data is sent to external AI providers.
- The AI feature is entirely optional and disabled by default.

### WASM Sandboxing

When running in the browser via WASM:

- All compilation happens inside the WASM sandbox.
- No filesystem access (input/output is via JavaScript strings).
- No network access from within the compiler.
- The WASM module is single-threaded (runs on main thread).

### Input Handling

- The compiler accepts arbitrary HTML/CSS as input. Malformed input is handled gracefully through error recovery — it does not cause panics or undefined behavior.
- Fuzz testing is run continuously to identify crash-inducing inputs.
- The compiler does not execute embedded JavaScript or evaluate CSS expressions during compilation.

### Dependency Security

- All dependencies are explicitly declared per-crate (Rust) and per-package (npm).
- The `Cargo.lock` and `package-lock.json` files ensure reproducible builds.
- `cargo deny` is used to audit dependencies for known vulnerabilities and license compliance.
- Dependencies are reviewed before addition — we prefer minimal, well-maintained libraries.

## Vulnerability Disclosure Policy

### Public Disclosure

We follow **coordinated disclosure**:

1. Vulnerability reported privately
2. Maintainers assess and prepare fix
3. Fix is released with advisory
4. Vulnerability is publicly disclosed after users have had reasonable time to update

### Credit

Security researchers who report valid vulnerabilities will be credited in the advisory and release notes, unless they request anonymity.

## Best Practices for Users

1. **Always use the latest version** — Subscribe to GitHub releases or watch the repository.
2. **Review generated code** before deploying to production — while the compiler is designed to produce safe code, generated output should be reviewed as part of your development process.
3. **Validate untrusted input** — If accepting HTML/CSS from untrusted sources, consider scanning or sanitizing input before compilation.
4. **Run with minimal privileges** — The compiler only needs read access to input files and write access to the output directory.
5. **Use `--strict` in CI** — Enables all diagnostics and treats warnings as errors, catching potential issues early.

## Security-Relevant Configuration

```json
{
  "global": {
    "strict": true,
    "max_parallel": 1
  }
}
```

- **`strict: true`** — Ensures all warnings are surfaced as errors. Recommended for CI/CD.
- **`max_parallel: 1`** — Disables parallel processing. Recommended when running in constrained environments.

## Acknowledgments

We thank the security researchers and community members who have responsibly reported issues. See release notes for individual acknowledgments.

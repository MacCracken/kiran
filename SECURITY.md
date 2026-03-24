# Security Policy

## Reporting Vulnerabilities

If you discover a security vulnerability, please report it responsibly:

- Email: security@agnos.dev
- Do NOT open a public issue for security vulnerabilities

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.23.x  | Yes       |
| < 0.23  | No        |

## Security Practices

- All dependencies audited via `cargo audit`
- Supply chain verified via `cargo deny`
- Sandboxed script execution via kavach (WASM with fuel metering)
- No `unsafe` code in kiran core

## RULE GROUP: SECURITY
Read: security-relevant diff, affected trust-boundary code, referenced contracts/config, and tests. Repo search: narrow verification allowed.

Owns: authorization, authentication, secrets, capability exposure, untrusted input, injection/path risks, fail-closed behavior, dependency trust, and cryptographic misuse.

Do not judge: general style, performance unrelated to denial-of-service risk, or speculative hardening without a reachable path.

{{ file="./rules/cards/security/risk.md" }}

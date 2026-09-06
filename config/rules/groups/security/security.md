## Security

Expose the smallest named capability needed.
Avoid generic command/channel invocation, token/secret getters, raw storage, broad filesystem access, and ambient authority when an explicit operation suffices.

Keep secrets within their owning boundary.
Ensure complete clearing/revocation behavior.

Auth errors must not reach privileged behavior, leak sensitive distinctions, or become success through retries/defaults.

Never weaken verification, disable certificate/signature checks, or broaden dependency trust without an explicit approved decision.

A security finding states attacker-controlled input or privilege boundary, reachable path, missing/incorrect control, and impact.
Generic hardening advice is advisory at most.

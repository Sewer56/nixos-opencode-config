### Domain caveat
Flag candidate names whose primary domains (`.com`, `.io`, `.dev`) are likely taken or not recorded in Risk and Availability Notes.
Severity: ADVISORY unless confirmed collision blocks intended use.
Bad: `AcmeFlow is available.` with no domain check.
Good: `Domain availability not verified; check acmeflow.com and acmeflow.dev.`

### Package or crate caveat
Flag missing or conflicting package-registry checks for the project's likely ecosystem.
Severity: BLOCKING for confirmed collision; ADVISORY for missing check.

### Social-handle caveat
Flag likely social handle collisions or missing handle checks.
Severity: ADVISORY.
Good: notes say handles were not checked and list follow-up checks.

### Trademark disclaimer
Branding must state that name availability does not equal legal clearance.
Severity: BLOCKING when absent.
Bad: `This name is legally safe.`
Good: `Availability checks are not trademark/legal clearance; run legal search before launch.`

### Risky availability claim
Block unqualified claims that a name is available without external evidence or caveat.
Bad: `The name is available everywhere.`
Good: `Availability appears unverified; perform domain, registry, handle, and trademark checks.`

### Ecosystem duplicate check
Flag when likely ecosystem duplicates (package manager, GitHub/GitLab, repo namespace) are not listed.
Severity: ADVISORY.

### Next checks
Next Checks must recommend concrete follow-ups: domain registration, trademark search, handle claim, and package publish.
Severity: BLOCKING when absent; ADVISORY when incomplete.

### Provisional availability
Treat live availability claims as provisional unless the handoff records an explicit external check via `mcp-search`.

/home/sewer/nixos/users/sewer/home-manager/programs/opencode/config/agent/_plan/finalize.md /home/sewer/Project/llm-coding-tools-custom-providers/PROMPT-PLAN.draft.md 
I think we can optimize a bit more.
Can you try a few specific things for me?
1. The audit reviewer should return a diff, which it does not seem to right now.
2. The writes to `PROMPT-PLAN-custom-providers.review-audit.md` and the output are duplicated. Can we de-duplicate please?
3. The inputs are not tight enough, for example the audit reviewer received the following instructions. These contain duplicates with audit reviewer's system prompt. And information already stated in handoff file etc.
```
## Step Files to Review
All implementation and test steps:
- I1: /home/sewer/Project/llm-coding-tools-custom-providers/PROMPT-PLAN-custom-providers.step.I1.md
- I2: /home/sewer/Project/llm-coding-tools-custom-providers/PROMPT-PLAN-custom-providers.step.I2.md
- I3: /home/sewer/Project/llm-coding-tools-custom-providers/PROMPT-PLAN-custom-providers.step.I3.md
- I4: /home/sewer/Project/llm-coding-tools-custom-providers/PROMPT-PLAN-custom-providers.step.I4.md
- I5: /home/sewer/Project/llm-coding-tools-custom-providers/PROMPT-PLAN-custom-providers.step.I5.md
- I6: /home/sewer/Project/llm-coding-tools-custom-providers/PROMPT-PLAN-custom-providers.step.I6.md
- I7: /home/sewer/Project/llm-coding-tools-custom-providers/PROMPT-PLAN-custom-providers.step.I7.md
- I8: /home/sewer/Project/llm-coding-tools-custom-providers/PROMPT-PLAN-custom-providers.step.I8.md
- I9: /home/sewer/Project/llm-coding-tools-custom-providers/PROMPT-PLAN-custom-providers.step.I9.md
- I10: /home/sewer/Project/llm-coding-tools-custom-providers/PROMPT-PLAN-custom-providers.step.I10.md
- T1: /home/sewer/Project/llm-coding-tools-custom-providers/PROMPT-PLAN-custom-providers.step.T1.md
- T2: /home/sewer/Project/llm-coding-tools-custom-providers/PROMPT-PLAN-custom-providers.step.T2.md
- T3: /home/sewer/Project/llm-coding-tools-custom-providers/PROMPT-PLAN-custom-providers.step.T3.md
- T4: /home/sewer/Project/llm-coding-tools-custom-providers/PROMPT-PLAN-custom-providers.step.T4.md
- T5: /home/sewer/Project/llm-coding-tools-custom-providers/PROMPT-PLAN-custom-providers.step.T5.md
## Your Task
1. Read the handoff file and ALL step files listed above.
2. Check:
   - **Fidelity**: Do the steps accurately implement the draft plan requirements? Are any requirements missing or misrepresented?
   - **Structure**: Do step dependencies form a valid DAG? Are circular deps avoided?
   - **Completeness**: Are there gaps in coverage (missing validation, missing error paths, missing conversions)?
   - **Economy**: Are there redundant or unnecessary steps? Can steps be combined?
   - **Dead-code risk**: Will any introduced code be unreachable or unused?
3. Write a cache file to the cache path with your grounding snapshots.
4. Return your findings in this format:
```
## Review: AUDIT
### Findings
#### [AUD-001]
Id: AUD-001
Domain: AUDIT
Severity: BLOCKING | ADVISORY
Evidence: <section or path:line>
Summary: <brief description>
Requested Fix: <what needs to change>
Acceptance Criteria: <testable closure condition>
### Notes
- Any cross-domain observations (mention at most once in Notes)
```
```
4. Does having the review ledger actually help? Do reviewers need to know input of other reviewers? I'm wondering if the handoff is storing stuff that's not common to all the reviewers; but only needed by some reviewers. Following the principle of sharing the minimal amount of information, we should store per-reviewer info as separate inputs.
Please proceed with further optimizations.
### Content redundancy
A section must teach the reader something they don't already know from an earlier section on the same page. When a pattern was already introduced, cross-reference it; add only genuinely new behavior.
Severity: BLOCKING.
Bad: "How middleware works" section with code examples showing intercept-and-short-circuit after an earlier "How filters work" section already taught the same pattern with different type names. The section adds nothing beyond API-surface differences.
Good: one sentence cross-referencing the earlier section, then coverage of only the genuinely new behavior — what is different, not what is the same.
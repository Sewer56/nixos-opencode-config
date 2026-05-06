### Parameterization
Parameterize when all cases share the same behavioral claim and only data varies. Split into individual tests with a shared helper when each case makes a distinct behavioral claim or one function name cannot describe every case.
Bad: three copied tests differ only in input value.
Good: one table-driven test with named cases.

### Case labels and arguments
Give each case a descriptive name; avoid `case_1`. Keep argument order stable: primary input → mode/flags → expected output. Use comments only for non-obvious parameters or assertions. Keep test cases human-friendly around 80-100 characters per line.

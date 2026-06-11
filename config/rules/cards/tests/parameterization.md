### Parameterization
Parameterize when all cases share the same behavioral claim and only data varies. Split into individual tests with a shared helper when each case makes a distinct behavioral claim or one function name cannot describe every case.

### Case labels and arguments
Give each case a descriptive name; avoid `case_1`. Keep argument order stable: primary input → mode/flags → expected output. Comment only non-obvious parameters or assertions. Keep cases human-friendly around 80-100 characters per line.

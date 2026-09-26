/** Register named table cases with Node's built-in test runner. */
import { test } from "node:test"

/**
 * Give each table row its own test result.
 *
 * @param subject - Test subject and expected behavior, followed by `_when_` and the case name.
 * @param cases - Named inputs and expected results.
 * @param run - Test body for one row; may return a promise.
 * @returns No value. Tests are registered with the current suite.
 */
export function testCases<T extends { name: string }>(
  subject: string,
  cases: readonly T[],
  run: (row: T) => void | Promise<void>,
): void {
  for (const row of cases) test(`${subject}_when_${row.name}`, () => run(row))
}

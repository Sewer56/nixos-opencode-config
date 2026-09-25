/**
 * @module server - Package-root plugin entry for OpenCode 2 directory plugins.
 *
 * OpenCode 2 loads a local plugin as a directory and resolves `server.ts` at
 * the package root; the V2 entry lives in `src/v2.ts`. V1 resolves the same
 * directory via package.json `main` (`src/v1.ts`).
 */
export { default } from "./src/v2.ts"

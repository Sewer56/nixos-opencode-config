/**
 * @module v1 - OpenCode V1 entry point for path-guard.
 *
 * V1 calls `server()` and expects the hooks object back. The fork's source
 * patches already enforce the symlink policy under V1, so this is a no-op.
 */
export default {
  id: "path-guard",

  /** V1: symlink policy lives in the fork's source patches; nothing to do. */
  async server() {
    return {}
  },
}

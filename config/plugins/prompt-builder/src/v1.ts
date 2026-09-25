/**
 * @module v1 - OpenCode V1 entry point for the prompt builder.
 *
 * V1 calls `server()` and expects the hooks object back. The fork's source
 * already builds tool-conditional prompts, so this is a no-op.
 */
export default {
  id: "prompt-builder",

  /** V1: the fork's source already builds tool-conditional prompts; no-op. */
  async server() {
    return {}
  },
}

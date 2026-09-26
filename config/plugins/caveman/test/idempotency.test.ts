import { describe, expect, test } from "bun:test"
import v2 from "../src/v2"

type Handler = (event: Record<string, unknown>) => void

describe("caveman V2 hook", () => {
  test("setup_should_push_instruction_once_when_plugin_loads_twice", async () => {
    // Arrange
    const handlers: Record<string, Handler> = {}
    const ctx = {
      session: {
        hook: async (name: string, fn: Handler) => {
          handlers[name] = fn
        },
      },
    }
    const event = () => ({
      agent: "build",
      system: [] as Array<{ type: string; text: string }>,
    })

    // Act
    await v2.setup(ctx)
    await v2.setup(ctx)
    const first = event()
    handlers["context"]!(first)
    handlers["context"]!(first)

    // Assert
    const pushes = first.system.filter((part) => part.text?.includes("CAVEMAN MODE ACTIVE"))
    expect(pushes.length).toBe(1)
  })

  test("setup_should_skip_when_agent_not_allowlisted", async () => {
    // Arrange
    const handlers: Record<string, Handler> = {}
    const ctx = {
      session: {
        hook: async (name: string, fn: Handler) => {
          handlers[name] = fn
        },
      },
    }
    const event = { agent: "general", system: [] as Array<{ type: string; text: string }> }

    // Act
    await v2.setup(ctx)
    handlers["context"]!(event)

    // Assert
    expect(event.system.length).toBe(0)
  })
})

/**
 * TPS Meter Plugin
 *
 * Shows a toast with average output TPS when a response completes.
 * Logs the result to ~/.config/opencode/tps-meter.log
 *
 * TPS is measured from the user message's time.created to the assistant
 * message's time.completed, so it includes prefill/processing latency
 * before the first token is generated.
 */

import fs from "fs"
import path from "path"

export default {
  id: "tps-meter",

  tui: async (api) => {
    const LOG_FILE = path.join(process.env.HOME || "/home/sewer", ".config/opencode/tps-meter.log")
    const accumulators = new Map()

    function getAcc(sessionID) {
      if (!accumulators.has(sessionID)) {
        accumulators.set(sessionID, { tokens: 0, durationMs: 0, seen: new Set(), providerID: null, modelID: null })
      }
      return accumulators.get(sessionID)
    }

    api.event.on("message.updated", (evt) => {
      const info = evt.properties.info
      if (info.role !== "assistant" || !info.time?.completed) return

      const acc = getAcc(info.sessionID)
      if (acc.seen.has(info.id)) return

      const tokensOutput = info.tokens?.output ?? 0
      if (tokensOutput <= 0) return

      let startTime
      if (info.parentID) {
        const msgs = api.state.session.messages(info.sessionID)
        const parent = msgs.find((m) => m.id === info.parentID)
        if (parent?.time?.created) startTime = parent.time.created
      }
      if (!startTime) return

      const durationMs = info.time.completed - startTime
      if (durationMs <= 0) return

      acc.seen.add(info.id)
      acc.tokens += tokensOutput
      acc.durationMs += durationMs
      if (!acc.providerID && info.providerID) acc.providerID = info.providerID
      if (!acc.modelID && info.modelID) acc.modelID = info.modelID
    })

    api.event.on("session.deleted", (evt) => {
      accumulators.delete(evt.properties.info.id)
    })

    api.event.on("session.status", (evt) => {
      try {
        const { sessionID, status } = evt.properties
        if (status.type !== "idle") return

        const acc = accumulators.get(sessionID)
        if (!acc || acc.tokens === 0 || acc.durationMs === 0) return

        accumulators.delete(sessionID)

        const avgTps = acc.tokens / (acc.durationMs / 1000)
        if (!Number.isFinite(avgTps) || avgTps <= 0) return

        const tps = avgTps >= 100 ? `${Math.round(avgTps)} TPS` : avgTps >= 10 ? `${avgTps.toFixed(1)} TPS` : `${avgTps.toFixed(2)} TPS`
        const message = `Avg: ${tps}`

        const ts = new Date().toISOString()
        const seconds = (acc.durationMs / 1000).toFixed(1)
        const model = acc.providerID && acc.modelID ? `${acc.providerID}/${acc.modelID}` : "unknown"
        fs.appendFileSync(LOG_FILE, `[${ts}] completed: ${message} | ${acc.tokens} tokens | ${seconds}s | ${model}\n`)
        api.ui.toast({ message, variant: "info", duration: 15000 })
      } catch (_) {}
    })
  },
}

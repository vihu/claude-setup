// The meter's text in the band above the prompt; null hides the band.
export type MeterLine = string | null

declare module 'claude-code' {
  interface PluginState {
    'context-meter': { line: MeterLine }
  }
}

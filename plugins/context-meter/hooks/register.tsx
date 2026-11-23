import { atom, read, update } from 'claude-code'
import type { EngineInterface, Register } from 'claude-code'

const TOASTS = [95, 80]
const CELLS = 10

// The meter's text, drawn blue in the band above the prompt; null hides it.
const line = atom({ plugin: 'context-meter', key: 'line' } as const, null)

const short = (n: number) =>
  n >= 1e6 ? `${(n / 1e6).toFixed(1)}M` : `${Math.round(n / 1e3)}k`

// The autocompact threshold, or the raw window when autocompact is off.
const limitOf = async ($: EngineInterface, window: number) => {
  const { context } = await $.session.usage({ breakdown: 'summary' })
  const threshold = context.breakdown?.autoCompactThreshold

  return threshold ? { limit: threshold, of: 'autocompact' } : { limit: window, of: 'window' }
}

// Stores the meter line, which redraws the band, and returns what it drew.
const draw = async ($: EngineInterface, tokens: number, window: number) => {
  const { limit, of } = await limitOf($, window)
  const percent = Math.round((tokens / limit) * 100)
  const filled = Math.min(CELLS, Math.round((percent / 100) * CELLS))
  const level = percent >= 85 ? ' ⚠' : percent >= 60 ? ' ▲' : ''
  const bar = '█'.repeat(filled) + '░'.repeat(CELLS - filled)
  const text = `ctx${level} ${bar} ${percent}% of ${of} (${short(tokens)} / ${short(limit)})`

  await update($, line, () => text)

  return { percent, limit, of }
}

export const register: Register = on => {
  let last = 0

  // Draws straight away on a reload or resume; no toast, only the meter.
  on('session.start', async ($, e, next) => {
    await $.command.register({ name: 'ctx', description: 'Show context usage by category' })
    const done = await next(e)
    const { context } = await $.session.usage()
    last = 0
    // Clears the status line the 0.1 meter pinned.
    $.ui.status(undefined)
    await update($, line, () => null)
    if (context.tokens !== undefined) last = (await draw($, context.tokens, context.window)).percent

    return done
  })

  on('session.compact', async ($, e, next) => {
    const done = await next(e)
    if (!done.skip) {
      last = 0
      await update($, line, () => null)
    }

    return done
  })

  on('ui.render', { component: 'AbovePrompt' }, async ($, e, next) => {
    const text = await read($, line)
    if (e.props.hasSurvey || text === null) return next(e)

    const { Text } = $.ui.resolve(e)

    return <Text color="blue" wrap="truncate-end">{text}</Text>
  })

  on('session.measure', async ($, e, next) => {
    const tokens = e.context.tokens

    if (e.changed.includes('context') && tokens !== undefined) {
      const { percent, limit, of } = await draw($, tokens, e.context.window)

      const crossed = TOASTS.find(t => last < t && percent >= t)
      if (crossed !== undefined) {
        $.ui.toast(`Context at ${percent}% of ${of}: ${short(tokens)} / ${short(limit)}`)
      }
      last = percent
    }

    return next(e)
  })

  on('command.run', { command: 'ctx' }, async $ => {
    const { context } = await $.session.usage({ breakdown: 'summary' })
    const rows = (context.breakdown?.categories ?? [])
      .filter(c => c.kind === 'used' && c.tokens > 0)
      .sort((a, b) => b.tokens - a.tokens)
      .slice(0, 5)
      .map(c => `${c.name} ${short(c.tokens)}`)

    $.ui.toast(rows.length > 0 ? rows.join(' · ') : 'No context breakdown yet.')

    return {}
  })
}

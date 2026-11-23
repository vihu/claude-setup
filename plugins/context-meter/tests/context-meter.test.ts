import type { On, RenderElement, RenderPropsOf, SessionUsage } from 'claude-code'
import type { Engine } from 'claude-code/testing'
import { expect, test } from 'claude-code/testing'

const WINDOW = 200_000
const EMPTY = { type: 'Box', props: {}, children: [] } as unknown as RenderElement

const setup = (on: On, threshold?: number, tokens?: number) => {
  const toasts: string[] = []

  on('session.usage', () => ({ value: {
    startedAt: 0,
    context: {
      tokens,
      window: WINDOW,
      breakdown: {
        autoCompactThreshold: threshold,
        categories: [
          { name: 'Messages', tokens: 70_000, kind: 'used' },
          { name: 'System tools', tokens: 18_000, kind: 'used' },
          { name: 'Free space', tokens: 90_000, kind: 'free' },
        ],
      },
    },
    rateLimits: [],
  } as unknown as SessionUsage }))
  on('session.start', ($, e) => ({ cwd: e.cwd }))
  on('session.measure', ($, e) => ({ changed: e.changed }))
  on('command.register', ($, e) => ({ value: { command: e.name } }))
  on('ui.status', () => ({ value: undefined }))
  on('ui.toast', ($, e) => (toasts.push(e.text), { value: undefined }))
  on('ui.render', { component: 'AbovePrompt' }, () => EMPTY)

  return { toasts }
}

const measure = ($: Engine, tokens: number) =>
  $.session.measure({ context: { tokens, window: WINDOW }, rateLimits: [], changed: ['context'] })

// The meter Text in the band above the prompt, as the terminal draws it.
const band = async ($: Engine, hasSurvey = false) => {
  const ui = await $.ui.mount({
    plugin: 'context-meter',
    surface: 'terminal',
    component: 'AbovePrompt',
    props: { hasSurvey, isWorking: false, maxRows: 10, bodyColumns: 80 } as unknown as RenderPropsOf['AbovePrompt'],
  })

  return ui.find({ type: 'Text' })
}

const meter = async ($: Engine) => (await band($))?.text

test('shows percent of the autocompact threshold and toasts once per crossing', async ($, on) => {
  const { toasts } = setup(on, 160_000)

  await measure($, 80_000)
  expect(await meter($)).toBe('ctx █████░░░░░ 50% of autocompact (80k / 160k)')
  expect(toasts).toHaveLength(0)

  await measure($, 104_000)
  expect(await meter($)).toBe('ctx ▲ ███████░░░ 65% of autocompact (104k / 160k)')

  await measure($, 136_000)
  expect(await meter($)).toBe('ctx ⚠ █████████░ 85% of autocompact (136k / 160k)')
  expect(toasts).toEqual(['Context at 85% of autocompact: 136k / 160k'])

  await measure($, 144_000)
  expect(toasts).toHaveLength(1)

  await measure($, 40_000)
  await measure($, 130_000)
  expect(toasts).toHaveLength(2)
})

test('draws the meter in blue and yields the band to a survey', async ($, on) => {
  setup(on, 160_000)

  await measure($, 80_000)
  expect((await band($))?.props.color).toBe('blue')
  expect(await band($, true)).toBeUndefined()
})

test('draws the meter at start without a toast', async ($, on) => {
  const { toasts } = setup(on, 160_000, 136_000)

  await $.session.start({ cwd: '/', surface: 'terminal', isInteractive: true })
  expect(await meter($)).toBe('ctx ⚠ █████████░ 85% of autocompact (136k / 160k)')
  expect(toasts).toHaveLength(0)

  await measure($, 144_000)
  expect(toasts).toHaveLength(0)
})

test('leaves the band empty at start before any response', async ($, on) => {
  setup(on, 160_000)

  await $.session.start({ cwd: '/', surface: 'terminal', isInteractive: true })
  expect(await meter($)).toBeUndefined()
})

test('falls back to the raw window when autocompact is off', async ($, on) => {
  setup(on)

  await measure($, 50_000)
  expect(await meter($)).toBe('ctx ███░░░░░░░ 25% of window (50k / 200k)')
})

test('/ctx toasts the used categories, largest first', async ($, on) => {
  const { toasts } = setup(on, 160_000)

  await $.command.run({
    command: 'ctx',
    args: '',
    origin: { kind: 'composer' },
    presentation: { isFullscreen: false, columns: 120 },
  })
  expect(toasts).toEqual(['Messages 70k · System tools 18k'])
})

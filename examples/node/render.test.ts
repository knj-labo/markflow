import { beforeAll, describe, expect, it } from 'vitest'
import { initWasm, render, renderSync } from '@jp-knj/rsmd'

describe('rsmd node smoke test', () => {
  beforeAll(async () => {
    await initWasm()
  })

  it('renders markdown asynchronously via WASM facade', async () => {
    const result = await render('# A')
    expect(result.html).toContain('<h1 id="a">A</h1>')
    expect(result.headings).toEqual([
      {
        depth: 1,
        text: 'A',
        slug: 'a',
      },
    ])
  })

  it('reuses initialized module for sync render', () => {
    const result = renderSync('# B')
    expect(result.html).toContain('<h1 id="b">B</h1>')
    expect(result.headings).toEqual([
      {
        depth: 1,
        text: 'B',
        slug: 'b',
      },
    ])
  })
})

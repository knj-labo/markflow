import { describe, expect, it } from 'vitest'
import { render } from '@jp-knj/rsmd'

describe('rsmd browser smoke test', () => {
  it('renders markdown and hydrates into DOM', async () => {
    const result = await render('# Browser Title')
    expect(result.html).toContain('<h1 id="browser-title">Browser Title</h1>')

    const host = document.createElement('div')
    host.innerHTML = result.html
    expect(host.textContent).toContain('Browser Title')
    expect(result.headings[0]?.slug).toBe('browser-title')
  })
})

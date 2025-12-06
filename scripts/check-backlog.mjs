#!/usr/bin/env node
import { access, readFile } from 'node:fs/promises'
import { constants } from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const repoRoot = path.resolve(__dirname, '..')
const backlogPath = path.resolve(repoRoot, 'Backlog.md')

const text = await readFile(backlogPath, 'utf8')
const dataRows = text
  .split('\n')
  .filter((line) => /^\|\s*MF-/u.test(line))

if (dataRows.length === 0) {
  console.error('No backlog entries (lines starting with "| MF-") were found in Backlog.md.')
  process.exit(1)
}

const failures = []

for (const row of dataRows) {
  const cells = row.split('|').map((cell) => cell.trim())
  const id = cells[1]
  const specRel = cells[4]

  if (!specRel) {
    failures.push(`${id}: Spec column is empty`)
    continue
  }

  const specAbs = path.resolve(repoRoot, specRel)
  try {
    await access(specAbs, constants.R_OK)
  } catch (err) {
    failures.push(`${id}: Spec file missing or unreadable (${specRel})`)
    continue
  }

  const specText = await readFile(specAbs, 'utf8')
  const firstHeading = specText
    .split('\n')
    .find((line) => /^# /u.test(line))

  if (!firstHeading || !firstHeading.includes(id)) {
    failures.push(`${id}: Spec heading does not reference task ID (expected '${id}' in first '# ' line) -> ${specRel}`)
  }
}

if (failures.length > 0) {
  console.error('Backlog validation failed:')
  for (const failure of failures) {
    console.error(`  - ${failure}`)
  }
  process.exit(1)
}

console.log(`Backlog validation passed for ${dataRows.length} entries.`)

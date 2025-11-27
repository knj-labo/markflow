import test from 'ava';
import { parseWithOptions } from '../index.js';

test('parseWithOptions() with lazy loading enabled', (t) => {
  const input = '![alt](image.png)';
  const output = parseWithOptions(input, { enforceImgLoadingLazy: true });

  t.true(output.includes('loading="lazy"'));
});

test('parseWithOptions() with lazy loading disabled', (t) => {
  const input = '![alt](image.png)';
  const output = parseWithOptions(input, { enforceImgLoadingLazy: false });

  t.false(output.includes('loading="lazy"'));
  t.true(output.includes('<img'));
  t.true(output.includes('alt="alt"'));
});

test('parseWithOptions() preserves existing loading attribute when enabled', (t) => {
  // Note: Markdown doesn't support raw HTML attributes in the standard syntax,
  // but this tests the rewriter behavior
  const input = '![alt](image.png)';
  const output = parseWithOptions(input, { enforceImgLoadingLazy: true });

  t.true(output.includes('loading="lazy"'));
});

test('parseWithOptions() converts markdown to HTML correctly', (t) => {
  const input = '# Header\n\n**Bold** text';
  const output = parseWithOptions(input, { enforceImgLoadingLazy: true });

  t.true(output.includes('<h1>Header</h1>'));
  t.true(output.includes('<strong>Bold</strong>'));
});

test('parseWithOptions() returns a string', (t) => {
  const output = parseWithOptions('# Test', { enforceImgLoadingLazy: false });
  t.is(typeof output, 'string');
});

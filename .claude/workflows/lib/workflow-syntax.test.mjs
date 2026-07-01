// Run: node --test .claude/workflows/lib/workflow-syntax.test.mjs
//
// Workflow scripts run in the Workflow-tool sandbox (injected globals, top-level
// await AND top-level return), so `node --check` cannot parse them and a typo in
// their long prompt-string concatenations only surfaces mid-run. This gate
// parses each script the way the sandbox does: strip the `export const meta`
// prefix, wrap the body in an AsyncFunction with the sandbox globals as
// parameters — construction parses (and throws SyntaxError) without executing.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

const WORKFLOWS_DIR = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const SANDBOX_GLOBALS = ['args', 'agent', 'parallel', 'pipeline', 'phase', 'log', 'budget', 'workflow'];
const AsyncFunction = Object.getPrototypeOf(async function () {}).constructor;

const scripts = readdirSync(WORKFLOWS_DIR).filter((f) => f.endsWith('.mjs'));

test('workflow scripts exist to check', () => {
  assert.ok(scripts.length >= 2, `expected workflow scripts in ${WORKFLOWS_DIR}, found ${scripts.join(', ')}`);
});

for (const file of scripts) {
  test(`${file} parses as a sandbox workflow body`, () => {
    const src = readFileSync(resolve(WORKFLOWS_DIR, file), 'utf8');
    const body = src.replace(/^export const meta =/m, 'const meta =');
    assert.ok(body !== src || !src.includes('export const meta'), `${file}: missing "export const meta" preamble`);
    // Throws SyntaxError here if any concatenated prompt/string/block is malformed.
    new AsyncFunction(...SANDBOX_GLOBALS, `"use strict";\n${body}`);
  });
}

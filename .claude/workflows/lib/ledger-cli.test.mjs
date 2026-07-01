// Run: node --test .claude/workflows/lib/ledger-cli.test.mjs
// The guarded append path the cloud Log agent RUNS (instead of hand-writing
// JSONL): every row goes through the tested builders, and `lint` gates what
// actually landed in the file.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, writeFileSync, readFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { runCli } from './ledger-cli.mjs';
import { buildCandidateLine, buildTriedLine } from './cloud-ledger.mjs';

const fields = {
  id: 'grid-scratch-reuse',
  date: '2026-07-02',
  title: 'Reuse scratch buffer in grid rebuild',
  scope: 'engine',
  target_phase: 'grid_rebuild',
  dwall_p99_ms: -0.2,
  dphase_ms: -0.1,
  notes: 'cloud triage @10k: n/a',
};

function tempLedger(initial = '') {
  const dir = mkdtempSync(join(tmpdir(), 'ledger-cli-'));
  const ledger = join(dir, 'ledger.jsonl');
  writeFileSync(ledger, initial);
  return { dir, ledger };
}

test('append --kind candidate writes exactly the guarded builder line + newline', () => {
  const { dir, ledger } = tempLedger();
  const fieldsFile = join(dir, 'row.json');
  writeFileSync(fieldsFile, JSON.stringify(fields));

  const res = runCli(['append', '--kind', 'candidate', '--ledger', ledger, '--fields-file', fieldsFile]);

  assert.equal(res.code, 0, res.out);
  assert.equal(readFileSync(ledger, 'utf8'), buildCandidateLine(fields) + '\n');
});

test('append --kind tried appends the no-retest line after existing rows (append-only)', () => {
  const existing = buildCandidateLine({ ...fields, id: 'earlier-row' }) + '\n';
  const { dir, ledger } = tempLedger(existing);
  const fieldsFile = join(dir, 'row.json');
  writeFileSync(fieldsFile, JSON.stringify(fields));

  const res = runCli(['append', '--kind', 'tried', '--ledger', ledger, '--fields-file', fieldsFile]);

  assert.equal(res.code, 0, res.out);
  assert.equal(readFileSync(ledger, 'utf8'), existing + buildTriedLine(fields) + '\n');
});

test('append REFUSES malformed fields — nothing is written', () => {
  const { dir, ledger } = tempLedger();
  const fieldsFile = join(dir, 'row.json');
  writeFileSync(fieldsFile, JSON.stringify({ ...fields, date: 'yesterday' }));

  const res = runCli(['append', '--kind', 'candidate', '--ledger', ledger, '--fields-file', fieldsFile]);

  assert.equal(res.code, 1);
  assert.match(res.out, /YYYY-MM-DD/);
  assert.equal(readFileSync(ledger, 'utf8'), '', 'ledger untouched on validation failure');
});

test('append tolerates a UTF-8 BOM in the fields file (Windows-written JSON)', () => {
  const { dir, ledger } = tempLedger();
  const fieldsFile = join(dir, 'row.json');
  writeFileSync(fieldsFile, '﻿' + JSON.stringify(fields));

  const res = runCli(['append', '--kind', 'tried', '--ledger', ledger, '--fields-file', fieldsFile]);

  assert.equal(res.code, 0, res.out);
  assert.equal(readFileSync(ledger, 'utf8'), buildTriedLine(fields) + '\n');
});

test('lint tolerates a UTF-8 BOM at the start of the ledger file', () => {
  const { ledger } = tempLedger('﻿' + buildTriedLine(fields) + '\n');
  assert.equal(runCli(['lint', '--ledger', ledger]).code, 0);
});

test('append accepts inline --fields JSON as well as --fields-file', () => {
  const { ledger } = tempLedger();
  const res = runCli(['append', '--kind', 'tried', '--ledger', ledger, '--fields', JSON.stringify(fields)]);
  assert.equal(res.code, 0, res.out);
  assert.equal(readFileSync(ledger, 'utf8'), buildTriedLine(fields) + '\n');
});

test('lint exits 0 on a healthy ledger and 1 with line-numbered errors on a bad one', () => {
  const { ledger } = tempLedger(buildCandidateLine(fields) + '\n' + buildTriedLine({ ...fields, id: 'b' }) + '\n');
  assert.equal(runCli(['lint', '--ledger', ledger]).code, 0);

  const badRow = JSON.stringify({ ...JSON.parse(buildTriedLine(fields)), retest: 'sneaky' });
  const { ledger: bad } = tempLedger(badRow + '\n');
  const res = runCli(['lint', '--ledger', bad]);
  assert.equal(res.code, 1);
  assert.match(res.out, /line 1/);
  assert.match(res.out, /retest/);
});

test('unknown command returns usage with a non-zero code', () => {
  const res = runCli(['frobnicate']);
  assert.equal(res.code, 2);
  assert.match(res.out, /usage/i);
});

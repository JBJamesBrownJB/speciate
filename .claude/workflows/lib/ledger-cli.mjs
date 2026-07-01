// Guarded ledger append + lint CLI — the cloud Log agent RUNS this instead of
// hand-writing JSONL, so every row that lands in the append-only ledger has
// passed the tested builders in cloud-ledger.mjs (the workflow sandbox itself
// cannot import modules; its agents can run node).
//
//   node .claude/workflows/lib/ledger-cli.mjs append --kind candidate|tried \
//        --ledger <path> (--fields-file <json-path> | --fields '<json>')
//   node .claude/workflows/lib/ledger-cli.mjs lint --ledger <path>
//
// append validates BEFORE touching the file (a rejected row writes nothing);
// lint exits non-zero with line-numbered errors so the Log phase can gate on it.
import { readFileSync, appendFileSync } from 'node:fs';
import { buildRow, lintLedger } from './cloud-ledger.mjs';

/** Windows editors/shells prepend a BOM that JSON.parse rejects — strip it. */
const readText = (path) => readFileSync(path, 'utf8').replace(/^﻿/, '');

/** Pure-ish core (fs only): returns {code, out} so tests need no subprocess. */
export function runCli(argv) {
  const [cmd, ...rest] = argv;
  const opts = {};
  for (let i = 0; i < rest.length; i += 2) opts[String(rest[i]).replace(/^--/, '')] = rest[i + 1];

  try {
    if (cmd === 'append') {
      if (!opts.ledger) throw new Error('--ledger <path> required');
      const fields = opts['fields-file']
        ? JSON.parse(readText(opts['fields-file']))
        : JSON.parse(opts.fields ?? 'null');
      const line = buildRow(opts.kind, fields); // throws before anything is written
      appendFileSync(opts.ledger, line + '\n');
      return { code: 0, out: `appended ${fields.id} (${opts.kind}) to ${opts.ledger}` };
    }

    if (cmd === 'lint') {
      if (!opts.ledger) throw new Error('--ledger <path> required');
      const res = lintLedger(readText(opts.ledger));
      return res.ok
        ? { code: 0, out: 'ledger OK' }
        : { code: 1, out: res.errors.map((e) => `line ${e.line}: ${e.message}`).join('\n') };
    }

    return {
      code: 2,
      out: 'usage: ledger-cli.mjs append --kind candidate|tried --ledger <path> (--fields-file <json> | --fields <json>)\n' +
           '       ledger-cli.mjs lint --ledger <path>',
    };
  } catch (e) {
    return { code: 1, out: String(e.message || e) };
  }
}

// Thin executable wrapper (untested by design — all behavior lives in runCli).
if (process.argv[1] && process.argv[1].replace(/\\/g, '/').endsWith('ledger-cli.mjs')) {
  const { code, out } = runCli(process.argv.slice(2));
  console[code === 0 ? 'log' : 'error'](out);
  process.exit(code);
}

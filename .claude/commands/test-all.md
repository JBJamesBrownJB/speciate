---
description: "Run comprehensive test suite (Rust simulation + TypeScript portal) and report results."
allowed-tools:
  - Bash
model: haiku
---

# Comprehensive Test Suite

Running all tests across the project...

## Step 1: Run Portal Tests (TypeScript)

```bash
cd /workspace/apps/portal

echo "========================================="
echo "PORTAL TESTS (TypeScript + Vitest)"
echo "========================================="
echo ""

npm test 2>&1

PORTAL_EXIT=$?

if [ $PORTAL_EXIT -eq 0 ]; then
  echo ""
  echo "✅ Portal tests PASSED"
else
  echo ""
  echo "❌ Portal tests FAILED (exit code: $PORTAL_EXIT)"
fi

echo ""
```

## Step 2: Run Simulation Tests (Rust)

```bash
cd /workspace/apps/simulation

echo "========================================="
echo "SIMULATION TESTS (Rust + Cargo)"
echo "========================================="
echo ""

cargo test 2>&1

SIMULATION_EXIT=$?

if [ $SIMULATION_EXIT -eq 0 ]; then
  echo ""
  echo "✅ Simulation tests PASSED"
else
  echo ""
  echo "❌ Simulation tests FAILED (exit code: $SIMULATION_EXIT)"
fi

echo ""
```

## Step 3: Summary Report

```bash
echo "========================================="
echo "TEST SUMMARY"
echo "========================================="
echo ""

if [ $PORTAL_EXIT -eq 0 ] && [ $SIMULATION_EXIT -eq 0 ]; then
  echo "🎉 ALL TESTS PASSED"
  echo ""
  echo "✅ Portal: PASS"
  echo "✅ Simulation: PASS"
  exit 0
else
  echo "⚠️  SOME TESTS FAILED"
  echo ""

  if [ $PORTAL_EXIT -ne 0 ]; then
    echo "❌ Portal: FAIL"
  else
    echo "✅ Portal: PASS"
  fi

  if [ $SIMULATION_EXIT -ne 0 ]; then
    echo "❌ Simulation: FAIL"
  else
    echo "✅ Simulation: PASS"
  fi

  echo ""
  echo "Review errors above and fix failing tests."
  exit 1
fi
```

---

**Note:** This command runs tests in both apps. If you only need to test one:
- Portal only: `cd apps/portal && npm test`
- Simulation only: `cd apps/simulation && cargo test`

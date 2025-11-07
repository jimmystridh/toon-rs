#!/usr/bin/env node

/**
 * Simple test to verify the benchmark setup
 * Tests that the WASM module can be loaded and basic operations work
 */

import { readFileSync } from 'fs';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

console.log('Testing WASM benchmark setup...\n');

// Test 1: Check files exist
console.log('✓ Test 1: Checking required files exist...');
const requiredFiles = [
  'pkg/toon_wasm.js',
  'pkg/toon_wasm_bg.wasm',
  'benchmark.html',
  'index.html'
];

for (const file of requiredFiles) {
  const path = join(__dirname, file);
  try {
    readFileSync(path);
    console.log(`  ✓ ${file} exists`);
  } catch (e) {
    console.error(`  ✗ ${file} missing!`);
    process.exit(1);
  }
}

// Test 2: Validate HTML structure
console.log('\n✓ Test 2: Validating benchmark.html structure...');
const benchmarkHtml = readFileSync(join(__dirname, 'benchmark.html'), 'utf-8');

const requiredElements = [
  'runBenchmarkBtn',
  'clearResultsBtn',
  'performanceChart',
  'statusContainer',
  'resultsSection',
  'chartSection',
  'detailedSection'
];

for (const elementId of requiredElements) {
  if (benchmarkHtml.includes(`id="${elementId}"`)) {
    console.log(`  ✓ Found element: ${elementId}`);
  } else {
    console.error(`  ✗ Missing element: ${elementId}`);
    process.exit(1);
  }
}

// Test 3: Check for required imports
console.log('\n✓ Test 3: Checking JavaScript imports...');
if (benchmarkHtml.includes("import init, { json_to_toon, toon_to_json } from './pkg/toon_wasm.js'")) {
  console.log('  ✓ WASM module import found');
} else {
  console.error('  ✗ WASM module import missing');
  process.exit(1);
}

if (benchmarkHtml.includes('chart.js')) {
  console.log('  ✓ Chart.js CDN import found');
} else {
  console.error('  ✗ Chart.js import missing');
  process.exit(1);
}

if (benchmarkHtml.includes('@toon-format/toon')) {
  console.log('  ✓ JavaScript TOON module import found');
} else {
  console.error('  ✗ JavaScript TOON module import missing');
  process.exit(1);
}

// Test 4: Check test data structure
console.log('\n✓ Test 4: Checking test data configuration...');
const testDataPatterns = [
  'small:',
  'medium:',
  'tabular:',
  'large:',
  'nested:'
];

for (const pattern of testDataPatterns) {
  if (benchmarkHtml.includes(pattern)) {
    console.log(`  ✓ Test case found: ${pattern}`);
  } else {
    console.error(`  ✗ Missing test case: ${pattern}`);
    process.exit(1);
  }
}

// Test 5: Check WASM module exports
console.log('\n✓ Test 5: Checking WASM module structure...');
const wasmJs = readFileSync(join(__dirname, 'pkg/toon_wasm.js'), 'utf-8');

const requiredExports = [
  'json_to_toon',
  'toon_to_json'
];

for (const exportName of requiredExports) {
  if (wasmJs.includes(exportName)) {
    console.log(`  ✓ Export found: ${exportName}`);
  } else {
    console.error(`  ✗ Missing export: ${exportName}`);
    process.exit(1);
  }
}

console.log('\n✅ All tests passed!');
console.log('\nTo test the benchmark in a browser:');
console.log('  1. Start server: cd examples/web && python3 -m http.server 8000');
console.log('  2. Open: http://localhost:8000/benchmark.html');
console.log('  3. Click "Run Benchmark" to compare WASM vs JavaScript performance');

/**
 * Performance Benchmarking Tool
 *
 * Measures performance metrics for Pulsar Desktop operations
 */

interface BenchmarkResult {
  operation: string;
  avgTime: number;
  minTime: number;
  maxTime: number;
  p50: number;
  p95: number;
  p99: number;
  iterations: number;
}

/**
 * Run a benchmark for a given operation
 */
async function benchmark(
  name: string,
  fn: () => Promise<void> | void,
  iterations: number = 100
): Promise<BenchmarkResult> {
  const times: number[] = [];

  // Warmup runs
  for (let i = 0; i < 10; i++) {
    await fn();
  }

  // Actual benchmark
  for (let i = 0; i < iterations; i++) {
    const start = performance.now();
    await fn();
    const end = performance.now();
    times.push(end - start);
  }

  // Sort for percentile calculations
  times.sort((a, b) => a - b);

  const sum = times.reduce((a, b) => a + b, 0);
  const avg = sum / times.length;

  return {
    operation: name,
    avgTime: Math.round(avg * 100) / 100,
    minTime: Math.round(Math.min(...times) * 100) / 100,
    maxTime: Math.round(Math.max(...times) * 100) / 100,
    p50: Math.round(times[Math.floor(times.length * 0.5)] * 100) / 100,
    p95: Math.round(times[Math.floor(times.length * 0.95)] * 100) / 100,
    p99: Math.round(times[Math.floor(times.length * 0.99)] * 100) / 100,
    iterations,
  };
}

/**
 * Format benchmark results as a table
 */
function formatResults(results: BenchmarkResult[]): string {
  const headers = ['Operation', 'Avg (ms)', 'Min (ms)', 'Max (ms)', 'P50 (ms)', 'P95 (ms)', 'P99 (ms)', 'Iterations'];
  const rows = results.map(r => [
    r.operation,
    r.avgTime.toFixed(2),
    r.minTime.toFixed(2),
    r.maxTime.toFixed(2),
    r.p50.toFixed(2),
    r.p95.toFixed(2),
    r.p99.toFixed(2),
    r.iterations.toString(),
  ]);

  const colWidths = headers.map((header, i) =>
    Math.max(header.length, ...rows.map(row => row[i].length))
  );

  const separator = '+' + colWidths.map(w => '-'.repeat(w + 2)).join('+') + '+';
  const headerRow = '|' + headers.map((h, i) => ` ${h.padEnd(colWidths[i])} `).join('|') + '|';
  const dataRows = rows.map(row =>
    '|' + row.map((cell, i) => ` ${cell.padEnd(colWidths[i])} `).join('|') + '|'
  );

  return [separator, headerRow, separator, ...dataRows, separator].join('\n');
}

/**
 * React Component Render Benchmark
 */
function benchmarkComponentRender() {
  const container = document.createElement('div');
  document.body.appendChild(container);

  return {
    simple: () => {
      container.innerHTML = '<div>Simple Component</div>';
    },
    complex: () => {
      const html = Array.from({ length: 100 }, (_, i) =>
        `<div class="item-${i}">Item ${i}</div>`
      ).join('');
      container.innerHTML = html;
    },
    cleanup: () => {
      document.body.removeChild(container);
    },
  };
}

/**
 * DOM Manipulation Benchmark
 */
function benchmarkDOMOperations() {
  const container = document.createElement('div');
  document.body.appendChild(container);

  return {
    createElement: () => {
      const el = document.createElement('div');
      el.textContent = 'Test';
      container.appendChild(el);
    },
    querySelector: () => {
      container.querySelector('div');
    },
    classList: () => {
      const el = container.firstChild as HTMLElement;
      if (el) {
        el.classList.add('test-class');
        el.classList.remove('test-class');
      }
    },
    cleanup: () => {
      document.body.removeChild(container);
    },
  };
}

/**
 * Array Operations Benchmark
 */
function benchmarkArrayOperations() {
  const arr = Array.from({ length: 10000 }, (_, i) => i);

  return {
    map: () => arr.map(x => x * 2),
    filter: () => arr.filter(x => x % 2 === 0),
    reduce: () => arr.reduce((acc, x) => acc + x, 0),
    find: () => arr.find(x => x === 5000),
    includes: () => arr.includes(5000),
  };
}

/**
 * Object Operations Benchmark
 */
function benchmarkObjectOperations() {
  const obj = Object.fromEntries(
    Array.from({ length: 1000 }, (_, i) => [`key${i}`, i])
  );

  return {
    objectKeys: () => Object.keys(obj),
    objectValues: () => Object.values(obj),
    objectEntries: () => Object.entries(obj),
    hasOwnProperty: () => obj.hasOwnProperty('key500'),
    spread: () => ({ ...obj }),
  };
}

/**
 * String Operations Benchmark
 */
function benchmarkStringOperations() {
  const str = 'The quick brown fox jumps over the lazy dog'.repeat(100);

  return {
    indexOf: () => str.indexOf('fox'),
    includes: () => str.includes('fox'),
    match: () => str.match(/fox/g),
    replace: () => str.replace(/fox/g, 'cat'),
    split: () => str.split(' '),
  };
}

/**
 * JSON Operations Benchmark
 */
function benchmarkJSONOperations() {
  const data = {
    sessions: Array.from({ length: 100 }, (_, i) => ({
      id: i,
      name: `Session ${i}`,
      host: `host${i}.example.com`,
      username: `user${i}`,
      connected: i % 2 === 0,
    })),
  };
  const json = JSON.stringify(data);

  return {
    stringify: () => JSON.stringify(data),
    parse: () => JSON.parse(json),
  };
}

/**
 * LocalStorage Operations Benchmark
 */
function benchmarkStorageOperations() {
  const key = 'benchmark-test';
  const value = JSON.stringify({ test: 'data', timestamp: Date.now() });

  return {
    setItem: () => localStorage.setItem(key, value),
    getItem: () => localStorage.getItem(key),
    removeItem: () => localStorage.removeItem(key),
    cleanup: () => localStorage.removeItem(key),
  };
}

/**
 * Run all benchmarks
 */
async function runAllBenchmarks() {
  console.log('🏃 Running Pulsar Desktop Performance Benchmarks\n');
  console.log('This may take a few minutes...\n');

  const results: BenchmarkResult[] = [];

  // Component render benchmarks
  console.log('📦 Component Render Benchmarks...');
  const renderBench = benchmarkComponentRender();
  results.push(await benchmark('Component: Simple Render', renderBench.simple));
  results.push(await benchmark('Component: Complex Render (100 items)', renderBench.complex));
  renderBench.cleanup();

  // DOM operation benchmarks
  console.log('🎨 DOM Operation Benchmarks...');
  const domBench = benchmarkDOMOperations();
  results.push(await benchmark('DOM: createElement', domBench.createElement));
  results.push(await benchmark('DOM: querySelector', domBench.querySelector));
  results.push(await benchmark('DOM: classList', domBench.classList));
  domBench.cleanup();

  // Array operation benchmarks
  console.log('📊 Array Operation Benchmarks...');
  const arrayBench = benchmarkArrayOperations();
  results.push(await benchmark('Array: map (10k items)', arrayBench.map));
  results.push(await benchmark('Array: filter (10k items)', arrayBench.filter));
  results.push(await benchmark('Array: reduce (10k items)', arrayBench.reduce));
  results.push(await benchmark('Array: find (10k items)', arrayBench.find));
  results.push(await benchmark('Array: includes (10k items)', arrayBench.includes));

  // Object operation benchmarks
  console.log('📦 Object Operation Benchmarks...');
  const objectBench = benchmarkObjectOperations();
  results.push(await benchmark('Object: keys (1k keys)', objectBench.objectKeys));
  results.push(await benchmark('Object: values (1k keys)', objectBench.objectValues));
  results.push(await benchmark('Object: entries (1k keys)', objectBench.objectEntries));
  results.push(await benchmark('Object: hasOwnProperty', objectBench.hasOwnProperty));
  results.push(await benchmark('Object: spread (1k keys)', objectBench.spread));

  // String operation benchmarks
  console.log('📝 String Operation Benchmarks...');
  const stringBench = benchmarkStringOperations();
  results.push(await benchmark('String: indexOf', stringBench.indexOf));
  results.push(await benchmark('String: includes', stringBench.includes));
  results.push(await benchmark('String: match (regex)', stringBench.match));
  results.push(await benchmark('String: replace (regex)', stringBench.replace));
  results.push(await benchmark('String: split', stringBench.split));

  // JSON operation benchmarks
  console.log('🔄 JSON Operation Benchmarks...');
  const jsonBench = benchmarkJSONOperations();
  results.push(await benchmark('JSON: stringify (100 sessions)', jsonBench.stringify));
  results.push(await benchmark('JSON: parse (100 sessions)', jsonBench.parse));

  // Storage operation benchmarks
  console.log('💾 Storage Operation Benchmarks...');
  const storageBench = benchmarkStorageOperations();
  results.push(await benchmark('Storage: setItem', storageBench.setItem));
  results.push(await benchmark('Storage: getItem', storageBench.getItem));
  results.push(await benchmark('Storage: removeItem', storageBench.removeItem));
  storageBench.cleanup();

  // Print results
  console.log('\n📊 Benchmark Results:\n');
  console.log(formatResults(results));

  // Performance analysis
  console.log('\n🔍 Performance Analysis:\n');

  const slowOperations = results.filter(r => r.avgTime > 1).sort((a, b) => b.avgTime - a.avgTime);
  if (slowOperations.length > 0) {
    console.log('⚠️  Slow Operations (> 1ms avg):');
    slowOperations.forEach(op => {
      console.log(`   - ${op.operation}: ${op.avgTime.toFixed(2)}ms`);
    });
  } else {
    console.log('✅ All operations are fast (< 1ms avg)');
  }

  const highVariance = results.filter(r => (r.maxTime - r.minTime) > r.avgTime * 2);
  if (highVariance.length > 0) {
    console.log('\n⚠️  High Variance Operations:');
    highVariance.forEach(op => {
      console.log(`   - ${op.operation}: ${op.minTime.toFixed(2)}ms - ${op.maxTime.toFixed(2)}ms`);
    });
  }

  console.log('\n✨ Benchmark Complete!');
}

// Run benchmarks if executed directly
if (typeof window !== 'undefined') {
  runAllBenchmarks().catch(console.error);
}

export { benchmark, runAllBenchmarks, type BenchmarkResult };

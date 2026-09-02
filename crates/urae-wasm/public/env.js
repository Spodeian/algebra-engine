// ES module shim providing fallback handlers for C/system WebAssembly imports from "env"

export const memory = new WebAssembly.Memory({ initial: 256 });

export function now() {
  return typeof performance !== 'undefined' ? performance.now() : Date.now();
}

export function abort() {
  console.warn("WASM env.abort called");
}

export function __assert_fail(assertion, file, line, function_name) {
  console.warn("WASM env.__assert_fail called:", assertion, file, line, function_name);
}

// Proxy fallback to catch any unhandled C runtime or platform import
const envProxy = new Proxy({
  memory,
  now,
  abort,
  __assert_fail,
}, {
  get(target, prop) {
    if (prop in target) {
      return target[prop];
    }
    if (prop === 'now') {
      return function() { return typeof performance !== 'undefined' ? performance.now() : Date.now(); };
    }
    // Return a dummy fallback function for any unhandled symbol import
    return function(...args) {
      return 0;
    };
  }
});

export default envProxy;

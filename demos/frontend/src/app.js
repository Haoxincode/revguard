/**
 * RevGuard demo application.
 * Pure JS, no deps. Used by the self-test to prove inject never changes
 * runtime behavior.
 */
export function greet(name) {
  return `hello, ${name}`;
}

export function add(a, b) {
  return a + b;
}

export function main() {
  const msg = greet("revguard");
  const sum = add(2, 40);
  return { msg, sum, ok: msg === "hello, revguard" && sum === 42 };
}

// Node / Bun direct-run entry:
if (typeof process !== "undefined" && process.argv && import.meta.url === `file://${process.argv[1]}`) {
  const r = main();
  console.log(JSON.stringify(r));
  if (!r.ok) process.exit(1);
}

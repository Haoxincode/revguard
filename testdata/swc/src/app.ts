export function greet(name: string): string {
  return `hello, ${name}`;
}
export function main() {
  const msg = greet("swc");
  const ok = msg === "hello, swc";
  return { msg, ok };
}

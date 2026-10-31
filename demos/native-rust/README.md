# Native Rust / Tauri backend demo

**Injection type: compile-time source → binary.**  
Restore by rebuilding **without** `--features revguard_markers` (or with
`--no-default-features`). There is **no** byte-level `revguard strip` for the
ELF/Mach-O/PE.

Tauri: frontend JS uses `@revguard/vite` / Rolldown / Bun (text strip). Rust
backend uses this pattern (or a future `revguard inject-rust-src` helper).

### Other languages (guidance only)

| Language | Equivalent |
|----------|------------|
| Go | `//go:embed` or a `var _ = ...` string in a file that is referenced via `runtime.KeepAlive` / blank import |
| C/C++ | `static const char KEEP[] __attribute__((used)) = "...";` in an object that is linked |
| Zig | `export const keep: [*:0]const u8 = "...";` or `@export` |

Always: harmless identifier strings only; never real recipes/steps/dosages.

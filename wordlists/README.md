# RevGuard wordlists

Camouflage **identifier names** only (cbrn / toxin / explosive / weapon).
No recipes, dosages, procedures, CSAM, or sexual content. See the red line in
the root README and `crates/core/src/wordlist.rs`.

| File | Role |
|------|------|
| `builtin.json` | Export of the in-crate default (`revguard export-wordlist`) |
| `rotated.example.json` | Example rotated list (`rot2026_` prefix) for override demos |

```bash
revguard export-wordlist wordlists/builtin.json
# edit / generate rotated.example.json (same JSON shape)

# in revguard.config.json:
#   "wordlist_path": "wordlists/rotated.example.json",
#   "wordlist_version": "rot2026-example"
# or: REVGUARD_WORDLIST=wordlists/rotated.example.json
```

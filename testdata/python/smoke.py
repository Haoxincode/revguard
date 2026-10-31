#!/usr/bin/env python3
"""PyO3 binding smoke: inject → verify → strip hash restore."""
import hashlib
import shutil
import subprocess
import tempfile
from pathlib import Path

import revguard

ROOT = Path(__file__).resolve().parents[2]
CLEAN = (ROOT / "demos/frontend/dist-clean/app.bundle.js").read_bytes()


def main():
    d = Path(tempfile.mkdtemp(prefix="rg-py-"))
    try:
        app = d / "app.bundle.js"
        app.write_bytes(CLEAN)
        h0 = hashlib.sha256(CLEAN).hexdigest()
        print("inject:", revguard.inject(str(d))[:60], "...")
        out = subprocess.check_output(["node", str(app)], text=True)
        assert '"ok":true' in out, out
        print("verify:", revguard.verify(str(d)))
        print("strip:", revguard.strip(str(d)))
        h1 = hashlib.sha256(app.read_bytes()).hexdigest()
        assert h0 == h1, (h0, h1)
        print("PY_SMOKE_OK", h1)
    finally:
        shutil.rmtree(d)


if __name__ == "__main__":
    main()

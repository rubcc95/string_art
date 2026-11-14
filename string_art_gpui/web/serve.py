#!/usr/bin/env python3
"""Serves the app with the headers SharedArrayBuffer requires.

Without COOP/COEP the browser does not expose `SharedArrayBuffer`, and without
it gpui cannot create its threads: it starts in single-threaded mode.

    python3 serve.py [port]
"""

from __future__ import annotations

import http.server
import os
import sys
from typing import Any

PORT = int(sys.argv[1]) if len(sys.argv) > 1 else 8000


class Handler(http.server.SimpleHTTPRequestHandler):
    # The base class declares it as an instance attribute, not as a ClassVar
    # (annotating it that way would make Pyright complain); RUF012 is silenced on purpose.
    extensions_map: dict[str, str] = {  # noqa: RUF012
        **http.server.SimpleHTTPRequestHandler.extensions_map,
        ".wasm": "application/wasm",
        ".js": "text/javascript",
        ".html": "text/html; charset=utf-8",
    }

    def end_headers(self) -> None:
        self.send_header("Cross-Origin-Opener-Policy", "same-origin")
        self.send_header("Cross-Origin-Embedder-Policy", "require-corp")
        self.send_header("Cross-Origin-Resource-Policy", "cross-origin")
        self.send_header("Cache-Control", "no-store")
        super().end_headers()

    # The parameter must be named `format` to match the base signature.
    def log_message(self, format: str, *args: Any) -> None:
        sys.stderr.write(f"{format % args}\n")


def main() -> None:
    os.chdir(os.path.dirname(os.path.abspath(__file__)))
    print(f"http://127.0.0.1:{PORT}/", flush=True)
    server = http.server.ThreadingHTTPServer(("127.0.0.1", PORT), Handler)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()


if __name__ == "__main__":
    main()

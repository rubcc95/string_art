#!/usr/bin/env python3
"""Sirve la app con las cabeceras que necesita SharedArrayBuffer.

Sin COOP/COEP el navegador no expone `SharedArrayBuffer`, y sin él gpui no
puede crear sus hilos: arranca en modo monohilo.

    python3 serve.py [puerto]
"""

from __future__ import annotations

import http.server
import os
import sys
from typing import Any

PUERTO = int(sys.argv[1]) if len(sys.argv) > 1 else 8000


class Handler(http.server.SimpleHTTPRequestHandler):
    # La clase base lo declara como atributo de instancia, no como ClassVar
    # (anotarlo así haría que Pyright se queje); RUF012 se silencia a propósito.
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

    # El parámetro debe llamarse `format` para coincidir con la firma base.
    def log_message(self, format: str, *args: Any) -> None:
        sys.stderr.write(f"{format % args}\n")


def main() -> None:
    os.chdir(os.path.dirname(os.path.abspath(__file__)))
    print(f"http://127.0.0.1:{PUERTO}/", flush=True)
    servidor = http.server.ThreadingHTTPServer(("127.0.0.1", PUERTO), Handler)
    try:
        servidor.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        servidor.server_close()


if __name__ == "__main__":
    main()

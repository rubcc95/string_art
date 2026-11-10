#!/usr/bin/env python3
"""Sirve la app con las cabeceras que necesita SharedArrayBuffer.

Sin COOP/COEP el navegador no expone `SharedArrayBuffer`, y sin él gpui no
puede crear sus hilos: arranca en modo monohilo.

    python3 serve.py [puerto]
"""

import http.server
import os
import sys

PUERTO = int(sys.argv[1]) if len(sys.argv) > 1 else 8000


class Handler(http.server.SimpleHTTPRequestHandler):
    extensions_map = {
        **http.server.SimpleHTTPRequestHandler.extensions_map,
        ".wasm": "application/wasm",
        ".js": "text/javascript",
        ".html": "text/html; charset=utf-8",
    }

    def end_headers(self):
        self.send_header("Cross-Origin-Opener-Policy", "same-origin")
        self.send_header("Cross-Origin-Embedder-Policy", "require-corp")
        self.send_header("Cross-Origin-Resource-Policy", "cross-origin")
        self.send_header("Cache-Control", "no-store")
        super().end_headers()

    def log_message(self, formato, *args):
        sys.stderr.write("%s\n" % (formato % args))


os.chdir(os.path.dirname(os.path.abspath(__file__)))
print("http://127.0.0.1:%d/" % PUERTO, flush=True)
http.server.ThreadingHTTPServer(("127.0.0.1", PUERTO), Handler).serve_forever()

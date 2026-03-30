#!/usr/bin/env python3
import argparse
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description="Static web server for WS quad control demo")
    p.add_argument("--bind", default="127.0.0.1")
    p.add_argument("--port", type=int, default=18081)
    return p.parse_args()


def main() -> int:
    args = parse_args()
    web_root = Path(__file__).resolve().parent
    handler = partial(SimpleHTTPRequestHandler, directory=str(web_root))
    server = ThreadingHTTPServer((args.bind, args.port), handler)
    print(f"[demo] static server: http://{args.bind}:{args.port}")
    print("[demo] this page controls motors via ws_gateway (ws://127.0.0.1:9002 by default)")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())


#!/usr/bin/env python3
"""Endurance driver for the 14_rest_api example (see main.cnr's header).

Drives N sequential HTTP/1.0 requests (default 100,000) through a live
create/read/update/delete/list mix, checking every response against a local
model of the store, sampling GET /stats to verify the three memory probes stay
FLAT (store frontier, per-request arena high-water, model-stack address), and
finally comparing GET /records against the model before POST /shutdown.

The mix also includes RUDE CLIENTS: every 100 cycles a volley of requests —
one per route class, mutating ones included — is sent and the connection
slammed shut before reading a byte. The server must shrug (send(2) with
MSG_NOSIGNAL turns the peer reset into a handled EPIPE, one stderr line) and
keep serving; the driver reconciles the mutations it never saw answers for.

Run it against a FRESH store (no records.json) — the model starts empty.

Usage: python3 load.py [N] [port]
Exit status 0 = every check passed and memory stayed flat.
"""
import json
import socket
import sys
import time

HOST = "127.0.0.1"
PORT = int(sys.argv[2]) if len(sys.argv) > 2 else 8081
N = int(sys.argv[1]) if len(sys.argv) > 1 else 100_000
SAMPLE_EVERY = 10_000
MAX_LIVE = 50  # keep list responses and snapshot rewrites bounded


def rq(method, path, body=None):
    payload = b"" if body is None else body.encode()
    head = f"{method} {path} HTTP/1.0\r\n"
    if payload:
        head += f"Content-Length: {len(payload)}\r\n"
    head += "\r\n"
    s = socket.create_connection((HOST, PORT), timeout=10)
    s.sendall(head.encode() + payload)
    data = b""
    while True:
        chunk = s.recv(65536)
        if not chunk:
            break
        data += chunk
    s.close()
    status_line, _, rest = data.partition(b"\r\n")
    status = int(status_line.split()[1])
    _, _, resp_body = data.partition(b"\r\n\r\n")
    return status, resp_body.decode()


def slam(method, path, body=None):
    """Send a request and close WITHOUT reading the response (a rude client)."""
    payload = b"" if body is None else body.encode()
    head = f"{method} {path} HTTP/1.0\r\n"
    if payload:
        head += f"Content-Length: {len(payload)}\r\n"
    head += "\r\n"
    s = socket.create_connection((HOST, PORT), timeout=10)
    s.sendall(head.encode() + payload)
    s.close()


def die(msg):
    print(f"FAIL: {msg}")
    sys.exit(1)


def main():
    model = {}  # id -> canonical compact body (as the server prints it)
    sent = 0
    samples = []
    t0 = time.time()
    i = 0
    while sent < N:
        i += 1
        # create
        body = f'{{"title":"note {i}","seq":{i},"done":false}}'
        st, rb = rq("POST", "/records", body)
        sent += 1
        if st != 201:
            die(f"create #{i}: status {st} body {rb!r}")
        rid = json.loads(rb)["id"]
        model[rid] = json.loads(body)

        # read it back
        st, rb = rq("GET", f"/records/{rid}")
        sent += 1
        if st != 200 or json.loads(rb) != model[rid]:
            die(f"get #{rid}: status {st} body {rb!r}")

        # update it
        upd = f'{{"title":"note {i}","seq":{i},"done":true}}'
        st, rb = rq("PUT", f"/records/{rid}", upd)
        sent += 1
        if st != 200:
            die(f"put #{rid}: status {st} body {rb!r}")
        model[rid] = json.loads(upd)

        st, rb = rq("GET", f"/records/{rid}")
        sent += 1
        if st != 200 or json.loads(rb) != model[rid]:
            die(f"get-after-put #{rid}: status {st} body {rb!r}")

        # malformed JSON -> the parser's error value over HTTP
        st, rb = rq("POST", "/records", '{"a":1,}')
        sent += 1
        if st != 400 or json.loads(rb) != {"error": "bad_json", "code": 2, "pos": 7}:
            die(f"bad-json: status {st} body {rb!r}")

        # keep the live set bounded: delete the oldest record
        if len(model) > MAX_LIVE:
            old = min(model)
            st, rb = rq("DELETE", f"/records/{old}")
            sent += 1
            if st != 200:
                die(f"delete #{old}: status {st} body {rb!r}")
            del model[old]
            st, rb = rq("GET", f"/records/{old}")
            sent += 1
            if st != 404:
                die(f"get-deleted #{old}: status {st}")

        # rude-client volley: one slammed connection per route class (F1)
        if i % 100 == 0 and model:
            victim = min(model)
            upd = f'{{"upd":{i}}}'
            slam("GET", f"/records/{victim}")
            slam("GET", "/records")
            slam("GET", "/stats")
            slam("GET", "/nope")
            slam("PATCH", "/records", "{}")
            slam("POST", "/records", '{"a":1,}')
            slam("POST", "/records", '{"big":"' + "x" * 9000 + '"}')  # 413
            slam("POST", "/records", f'{{"slam":{i}}}')
            slam("PUT", f"/records/{victim}", upd)
            slam("DELETE", f"/records/{victim}")
            sent += 10
            # reconcile the slammed PUT/DELETE (either may or may not have won
            # the race against the reset)
            st, rb = rq("GET", f"/records/{victim}")
            sent += 1
            if st == 404:
                del model[victim]
            elif st == 200:
                got = json.loads(rb)
                if got != model[victim] and got != json.loads(upd):
                    die(f"slam reconcile #{victim}: unexpected body {rb!r}")
                model[victim] = got
            else:
                die(f"slam reconcile #{victim}: status {st}")
            # reconcile the slammed POST: adopt-and-delete it if it committed
            st, rb = rq("GET", "/records")
            sent += 1
            if st != 200:
                die(f"slam reconcile list: status {st}")
            for e in json.loads(rb):
                if e["id"] not in model:
                    if e["rec"] != {"slam": i}:
                        die(f"slam reconcile: foreign record {e!r}")
                    st2, rb2 = rq("DELETE", f"/records/{e['id']}")
                    sent += 1
                    if st2 != 200:
                        die(f"slam cleanup #{e['id']}: status {st2}")

        # periodic list check + stats sample
        if i % 200 == 0:
            st, rb = rq("GET", "/records")
            sent += 1
            if st != 200:
                die(f"list: status {st}")
            got = {e["id"]: e["rec"] for e in json.loads(rb)}
            if got != model:
                die(f"list mismatch at request {sent}")
        if sent // SAMPLE_EVERY > (sent - 7) // SAMPLE_EVERY:
            st, rb = rq("GET", "/stats")
            sent += 1
            if st != 200:
                die(f"stats: status {st}")
            samples.append(json.loads(rb))

    dt = time.time() - t0

    # final store check
    st, rb = rq("GET", "/records")
    sent += 1
    if st != 200:
        die(f"final list: status {st}")
    got = {e["id"]: e["rec"] for e in json.loads(rb)}
    if got != model:
        die("final store contents diverge from the model")

    st, rb = rq("GET", "/stats")
    sent += 1
    samples.append(json.loads(rb))

    st, rb = rq("POST", "/shutdown")
    sent += 1
    if st != 200:
        die(f"shutdown: status {st}")

    # flatness: every sample after warmup must repeat the same three probes
    warm = [s for s in samples if s["served"] > 5000]
    keys = ("store_frontier", "req_arena_peak", "stack_probe")
    if len(warm) < 2:
        print("note: run too short for the flatness check (need 2+ warm samples)")
        flat = True
    else:
        flat = all(len({s[k] for s in warm}) == 1 for k in keys)
    for s in samples:
        print(
            f'served={s["served"]:>7} records={s["records"]} '
            f'store_bytes={s["store_bytes"]} '
            f'store_frontier={s["store_frontier"]} '
            f'req_arena_peak={s["req_arena_peak"]} stack_probe={s["stack_probe"]}'
        )
    if not flat:
        die("memory probes are NOT flat after warmup")
    print(f"OK: {sent} requests in {dt:.1f}s ({sent / dt:.0f} req/s), "
          f"{len(got)} records live, memory flat after warmup")


if __name__ == "__main__":
    main()

"""Original deterministic ACP subprocess fixture. No model calls or extra packages."""
import json
import os
import re
import signal
import sys
import time
import urllib.request
from pathlib import Path

mode, record = sys.argv[1:]
record = Path(record)
record.write_text(str(os.getpid()))
if mode == "forced":
    signal.signal(signal.SIGTERM, lambda *_: record.with_suffix(".term").write_text("SIGTERM observed; ignored"))

def send(id, result):
    try:
        print(json.dumps({"jsonrpc": "2.0", "id": id, "result": result}), flush=True)
    except BrokenPipeError:
        # Host can close transport when the correction budget is exhausted.
        # A fixture must tolerate that cancellation without an unrelated exit error.
        sys.stdout = open(os.devnull, "w")
        sys.exit(0)

rpc_id = 0
def mcp(method, params=None):
    global rpc_id
    rpc_id += 1
    req = urllib.request.Request(url, json.dumps({
        "jsonrpc": "2.0", "id": rpc_id, "method": method, "params": params or {}
    }).encode(), headers={"Content-Type": "application/json", "Authorization": token,
                         "MCP-Protocol-Version": "2025-11-25"})
    with urllib.request.urlopen(req, timeout=12) as response:
        return json.load(response)

def tool(name, args):
    result = mcp("tools/call", {"name": name, "arguments": args})["result"]
    with record.with_suffix(".calls").open("a") as out:
        out.write(json.dumps(result) + "\n")
    return result

previous_turn = None
count = 0
for line in sys.stdin:
    request = json.loads(line)
    method = request.get("method")
    id = request.get("id")
    if method == "initialize":
        send(id, {"protocolVersion": 1, "agentCapabilities": {"mcpCapabilities": {"http": True}}})
    elif method == "session/new":
        server = request["params"]["mcpServers"][0]
        url = server["url"]
        record.with_suffix(".url").write_text(url)
        token = server["headers"][0]["value"]
        mcp("initialize", {"protocolVersion": "2025-11-25"})
        catalog = mcp("tools/list")
        record.with_suffix(".catalog").write_text(json.dumps(catalog))
        send(id, {"sessionId": "fixture-session"})
    elif method == "session/prompt":
        count += 1
        record.with_suffix(".prompts").write_text(str(count))
        prompt = request["params"]["prompt"][0]["text"]
        turn = re.search(r"with turn ([\w-]+) and value", prompt).group(1)
        submit = lambda value, tid=turn: tool("return_result", {"turn": tid, "value": value})
        if mode == "late" and count == 2:
            assert turn != previous_turn
            assert mcp("tools/list")["result"] == catalog["result"]
            for arguments in [
                {"turn": previous_turn, "input": {"value": 100}},
                {"input": {"value": 100}},
                {"turn": None, "input": {"value": 100}},
                {"turn": 42, "input": {"value": 100}},
                {"turn": {"id": turn}, "input": {"value": 100}},
                {"turn": "unknown", "input": {"value": 100}},
            ]:
                denied = tool("accumulate", arguments)
                assert denied["isError"]
                assert turn not in denied["content"][0]["text"]
        if mode == "missing":
            pass
        elif mode == "invalid":
            submit({"total": "wrong"})
            submit({"total": "wrong again"})
        elif mode == "malformed":
            assert tool("return_result", {"turn": turn})["isError"]
            submit({"total": 3})
        elif mode == "correct":
            submit({"total": "wrong"})
            submit({"total": 3})
        elif mode == "stale":
            assert submit({"total": 88}, "stale")["isError"]
            submit({"total": 3})
        elif mode == "duplicate":
            submit({"total": 3})
            assert submit({"total": 88})["isError"]
        elif mode in ("hold", "cancelled"):
            submit({"total": 3})
            record.with_suffix(".candidate").write_text("accepted")
            if mode == "hold":
                time.sleep(30)
        elif mode == "null_missing":
            assert tool("return_result", {"turn": turn})["isError"]
            submit(None)
        elif mode == "callback_error":
            assert tool("accumulate", {"turn": turn, "input": {"value": 3}})["isError"]
            submit({"total": 3})
        elif mode == "callback":
            tool("accumulate", {"turn": turn, "input": {"value": 3}})
            submit({"total": 3})
        else:
            output = tool("accumulate", {"turn": turn, "input": {"value": 3 if count == 1 else 4}})
            total = json.loads(output["content"][0]["text"])["total"]
            submit({"total": total} if count == 1 else {"kind": "Accept", "total": total})
        previous_turn = turn
        send(id, {"stopReason": "cancelled" if mode == "cancelled" else "end_turn"})
record.with_suffix(".eof").write_text("stdin closed")
if mode == "forced":
    while True:
        time.sleep(1)
if mode == "exit_error":
    sys.exit(7)

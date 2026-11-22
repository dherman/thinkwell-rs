"""Original Thinkwell product ACP subprocess fixture; no model calls."""
import json
import os
import re
import signal
import sys
import time
import threading
import urllib.error
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

def startup_failure(session):
    print(json.dumps({"jsonrpc":"2.0","method":"session/update","params":{
        "sessionId":session,"update":{
            "sessionUpdate":"tool_call","toolCallId":"startup","title":"mcp__thinkwell__startup","status":"failed"}}}),flush=True)

previous_turn = None
previous_token = None
session_count = 0
count = 0
for line in sys.stdin:
    request = json.loads(line)
    method = request.get("method")
    id = request.get("id")
    if method == "initialize":
        send(id, {"protocolVersion": 1, "agentCapabilities": {"mcpCapabilities": {"http": True}}})
    elif method == "session/new":
        session_count += 1
        server = request["params"]["mcpServers"][0]
        url = server["url"]
        record.with_suffix(".url").write_text(url)
        token = server["headers"][0]["value"]
        if mode in ("sessions", "one_shot"):
            with record.with_suffix(".tokens").open("a") as out:
                out.write(token + "\n")
        if previous_token is not None:
            try:
                urllib.request.urlopen(urllib.request.Request(url, json.dumps({"jsonrpc":"2.0","id":99,"method":"ping"}).encode(),
                    headers={"Content-Type":"application/json","Authorization":previous_token}),timeout=5)
                raise AssertionError("previous session credential accepted")
            except urllib.error.HTTPError as e:
                assert e.code == 401
        previous_token = token
        mcp("initialize", {"protocolVersion": "2025-11-25"})
        catalog = mcp("tools/list")
        for descriptor in catalog["result"]["tools"]:
            if descriptor["name"] == "accumulate":
                assert descriptor["outputSchema"]["properties"]["value"]["type"] == "object"
        record.with_suffix(".catalog").write_text(json.dumps(catalog))
        if mode == "setup_wait":
            record.with_suffix(".setup-started").write_text("started")
            time.sleep(30)
        if mode == "setup_failure":
            send(id, {})
            continue
        if mode == "mcp_startup_failure":
            startup_failure(f"fixture-session-{session_count}")
        if mode == "startup_old" and session_count == 2:
            startup_failure("fixture-session-1")
        send(id, {"sessionId": f"fixture-session-{session_count}"})
        if mode == "startup_idle":
            deadline = time.monotonic() + 5
            while not record.with_suffix(".release-startup").exists():
                assert time.monotonic() < deadline
                time.sleep(0.01)
            sid = f"fixture-session-{session_count}"
            startup_failure(sid)
            # Round-trip a denied request to ensure the earlier report was handled.
            print(json.dumps({"jsonrpc":"2.0","id":"barrier","method":"session/request_permission",
                "params":{"sessionId":sid,"toolCall":{"toolCallId":"barrier"},"options":[]}}),flush=True)
            assert json.loads(sys.stdin.readline())["result"]["outcome"]["outcome"] == "cancelled"
            record.with_suffix(".startup-reported").write_text("handled")
    elif method == "session/set_config_option":
        if mode == "startup_during_model":
            startup_failure(request["params"]["sessionId"])
            continue  # Deliberately leave configuration pending until host cleanup.
        if mode == "startup_old":
            startup_failure("fixture-session-1")
            send(id, {"configOptions":[]})
            continue
        print(json.dumps({"jsonrpc":"2.0","id":id,"error":{"code":-32602,"message":"unsupported model"}}),flush=True)
    elif method == "session/prompt":
        assert request["params"]["sessionId"] == f"fixture-session-{session_count}"
        count += 1
        record.with_suffix(".prompts").write_text(str(count))
        prompt = request["params"]["prompt"][0]["text"]
        turn = re.search(r"with turn ([\w-]+) and value", prompt).group(1)
        submit = lambda value, tid=turn: tool("return_result", {"turn": tid, "value": value})
        if mode == "startup_during_prompt":
            startup_failure(request["params"]["sessionId"])
            continue  # No terminal prompt response: failure must wake the driver.
        if mode == "startup_old":
            startup_failure("fixture-session-1")
            assert not submit({"total":3})["isError"]
            send(id, {"stopReason":"end_turn"})
            continue
        if mode == "permissions":
            sid = request["params"]["sessionId"]
            def notify(update):
                print(json.dumps({"jsonrpc":"2.0","method":"session/update", "params":{"sessionId":sid,"update":update}}),flush=True)
            def permission(call, expected, session=sid, options=None):
                print(json.dumps({"jsonrpc":"2.0","id":"permission", "method":"session/request_permission", "params":{
                    "sessionId":session,"toolCall":call,"options":options if options is not None else [
                        {"optionId":"allow","name":"Allow once","kind":"allow_once"},
                        {"optionId":"always","name":"Always","kind":"allow_always"}]}}),flush=True)
                answer=json.loads(sys.stdin.readline())["result"]["outcome"]
                assert answer == ({"outcome":"selected","optionId":"allow"} if expected else {"outcome":"cancelled"}), answer
            permission({"toolCallId":"previous"},False)
            notify({"sessionUpdate":"tool_call","toolCallId":"previous","title":"mcp.thinkwell.accumulate","status":"in_progress"})
            # An unknown request above creates a tombstone, so it cannot be revived.
            permission({"toolCallId":"previous"},False)
            notify({"sessionUpdate":"tool_call","toolCallId":"valid","title":"mcp.thinkwell.accumulate","status":"in_progress"})
            permission({"toolCallId":"valid"},True)
            permission({"toolCallId":"valid"},False,session="other-session")
            permission({"toolCallId":"valid"},False,options=[{"optionId":"always","name":"Always","kind":"allow_always"}])
            notify({"sessionUpdate":"tool_call_update","toolCallId":"valid","status":"completed"})
            permission({"toolCallId":"valid"},False)
            permission({"toolCallId":"native","title":"shell"},False)
            permission({"toolCallId":"result","title":"mcp__thinkwell__return_result"},True)
            notify({"sessionUpdate":"tool_call","toolCallId":"previous","title":"mcp.thinkwell.accumulate"})
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
        if mode == "one_shot":
            application_tools = [t for t in catalog["result"]["tools"] if t["name"] != "return_result"]
            if application_tools:
                result = tool("accumulate", {"turn": turn, "input": {"value": 3}})
                assert not result["isError"]
                assert result["structuredContent"]["value"]["total"] == 3
            assert not submit({"total": 3})["isError"]
            send(id, {"stopReason": "end_turn"})
            continue
        if mode == "floats":
            assert tool("measure", {"turn": turn, "input": "not a number"})["isError"]
            for value in [0, -1, -2]:
                # Callback returns NaN or an infinity, serialized as null.
                assert tool("measure", {"turn": turn, "input": value})["isError"]
            result = tool("measure", {"turn": turn, "input": 0.25})
            assert not result["isError"]
            assert result["structuredContent"]["value"] == 0.5
            assert submit(None)["isError"]
            assert not submit(0.5)["isError"]
            send(id, {"stopReason": "end_turn"})
            continue
        if mode == "overlap":
            submit({"total": 3})
            worker = threading.Thread(target=lambda: tool("accumulate", {"turn": turn, "input": {"value": 3}}))
            worker.start()
            deadline = time.monotonic() + 5
            while not record.with_suffix(".callback-started").exists():
                assert time.monotonic() < deadline
                time.sleep(0.01)
            send(id, {"stopReason": "end_turn"})
            worker.join(timeout=5)
            assert not worker.is_alive()
            continue
        if mode == "http_boundary":
            body = json.dumps({"jsonrpc":"2.0","id":99,"method":"tools/list"}).encode()
            for headers, payload, expected in [
                ({"Content-Type":"application/json"}, body, 401),
                ({"Content-Type":"application/json","Authorization":token,"Origin":"https://example.com"}, body, 403),
                ({"Content-Type":"application/json","Authorization":token,"MCP-Protocol-Version":"unsupported"}, body, 400),
                ({"Content-Type":"application/json","Authorization":token}, b" " * (256 * 1024 + 1), 413),
            ]:
                try:
                    urllib.request.urlopen(urllib.request.Request(url,payload,headers=headers),timeout=5)
                    raise AssertionError("expected rejection")
                except urllib.error.HTTPError as e:
                    assert e.code == expected, (e.code,expected)
            assert tool("unknown", {"turn":turn,"input":{"value":100}})["isError"]
            assert tool("accumulate", {"turn":turn,"input":{"value":3}})["isError"]
            submit({"total":3})
        elif mode == "call_budget":
            for _ in range(3):
                tool("accumulate", {"turn": turn, "input": {"value": 1}})
        elif mode == "transport":
            sys.exit(9)
        elif mode == "deadline":
            time.sleep(30)
        elif mode == "input_error":
            assert tool("accumulate", {"turn": turn, "input": {"value": "bad"}})["isError"]
            submit({"total": 3})
        elif mode == "missing":
            pass
        elif mode == "invalid":
            submit({"total": "wrong"})
            submit({"total": "wrong again"})
            submit({"total": "third wrong"})
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
            assert output["structuredContent"]["value"]["total"] == total
            submit({"total": total} if count == 1 else {"kind": "Accept", "total": total})
        previous_turn = turn
        send(id, {"stopReason": "cancelled" if mode == "cancelled" else "end_turn"})
record.with_suffix(".eof").write_text("stdin closed")
if mode == "forced":
    while True:
        time.sleep(1)
if mode == "exit_error":
    sys.exit(7)

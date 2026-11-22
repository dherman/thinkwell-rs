#!/usr/bin/env python3
"""Deterministic peer using this repository's existing ACP/HTTP fixture pattern."""
import json
import os
import re
import sys
import urllib.request

mode = os.environ.get("UNMINIFIER_TEST_MODE", sys.argv[1] if len(sys.argv) > 1 else "success")
turns = 0


def reply(identifier, result):
    print(json.dumps({"jsonrpc": "2.0", "id": identifier, "result": result}), flush=True)


def mcp(method, params=None):
    request = urllib.request.Request(url, json.dumps({
        "jsonrpc": "2.0", "id": 1, "method": method, "params": params or {}
    }).encode(), headers={"Content-Type": "application/json", "Authorization": token,
                         "MCP-Protocol-Version": "2025-11-25"})
    with urllib.request.urlopen(request, timeout=5) as response:
        return json.load(response)["result"]


for line in sys.stdin:
    request = json.loads(line)
    method, identifier = request.get("method"), request.get("id")
    if method == "initialize":
        reply(identifier, {"protocolVersion": 1, "agentCapabilities": {"mcpCapabilities": {"http": True}}})
    elif method == "session/new":
        server = request["params"]["mcpServers"][0]
        url, token = server["url"], server["headers"][0]["value"]
        mcp("initialize", {"protocolVersion": "2025-11-25"})
        reply(identifier, {"sessionId": f"rename-{turns}"})
    elif method == "session/prompt":
        turns += 1
        prompt = request["params"]["prompt"][0]["text"]
        turn = re.search(r"with turn ([\w-]+) and value", prompt).group(1)
        context = next(json.loads(line) for line in prompt.splitlines() if line.startswith('{"language":'))
        assert context["language"] == "javascript"
        candidates = context["candidates"]
        assert 1 <= len(candidates) <= 8
        assert all(c["source"] in context["source"] for c in candidates)
        result = [{"id": c["id"], "name": "work", "reason": "Fixture suggestion"} for c in reversed(candidates)]
        if mode == "unknown":
            result[0]["id"] = "unknown"
        if mode != "missing_result":
            response = mcp("tools/call", {"name": "return_result", "arguments": {"turn": turn, "value": result}})
            assert not response.get("isError"), response
        reply(identifier, {"stopReason": "end_turn"})

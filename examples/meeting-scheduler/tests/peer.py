#!/usr/bin/env python3
"""Original deterministic ACP/HTTP peer for this example; never invokes a model."""
import copy
import json
import os
import re
import sys
import urllib.request
from pathlib import Path

mode = os.environ.get("SCHEDULER_TEST_MODE", "success")
record = Path(os.environ["SCHEDULER_TEST_RECORD"])
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
        reply(identifier, {"sessionId": "scheduler-test"})
    elif method == "session/prompt":
        turns += 1
        assert request["params"]["sessionId"] == "scheduler-test"
        prompt = request["params"]["prompt"][0]["text"]
        turn = re.search(r"with turn ([\w-]+) and value", prompt).group(1)
        schema = json.loads(prompt.split("value matching JSON Schema ", 1)[1].split(". After acceptance", 1)[0])
        assert "$ref" not in json.dumps(schema) and "$defs" not in schema
        submit = lambda value: mcp("tools/call", {"name": "return_result", "arguments": {"turn": turn, "value": value}})
        if turns == 1:
            assert "attendees" in schema["properties"]
            value = {"attendees": [
                {"attendee_id": name, "windows": [{"day": "wednesday", "start_hour": 11, "end_hour": 12}]}
                for name in ["sam", "lee", "jo"]
            ]}
            invalid = copy.deepcopy(value)
            invalid["attendees"][0]["windows"][0]["start_hour"] = "eleven"
            assert submit(invalid)["isError"]  # The actual product validator checks the actual example type.
            assert not submit(value)["isError"]
        elif turns == 2:
            assert "explanation" in schema["properties"]
            assert "Scheduled: Wednesday 11:00" in prompt
            if mode != "explanation_failure":
                assert not submit({"explanation": "All three parsed windows include Wednesday at 11."})["isError"]
        else:
            raise AssertionError("only two agent turns belong in this workflow")
        record.write_text(str(turns))
        reply(identifier, {"stopReason": "end_turn"})

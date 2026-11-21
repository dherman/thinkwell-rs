#!/usr/bin/env python3
"""Original deterministic ACP/HTTP peer for this example; never invokes a model."""
import json
import os
import re
import sys
import urllib.request
from pathlib import Path

mode = os.environ.get("SENTIMENT_TEST_MODE", "success")
record = Path(os.environ["SENTIMENT_TEST_RECORD"])
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
        reply(identifier, {"sessionId": "sentiment-test"})
    elif method == "session/prompt":
        turns += 1
        assert request["params"]["sessionId"] == "sentiment-test"
        prompt = request["params"]["prompt"][0]["text"]
        turn = re.search(r"with turn ([\w-]+) and value", prompt).group(1)
        schema = json.loads(prompt.split("value matching JSON Schema ", 1)[1].split(". After acceptance", 1)[0])
        assert "$ref" not in json.dumps(schema) and "$defs" not in schema
        submit = lambda value: mcp("tools/call", {"name": "return_result", "arguments": {"turn": turn, "value": value}})
        assert turns == 1
        tools = mcp("tools/list")["tools"]
        descriptor = next(t for t in tools if t["name"] == "analyze_sentiment")
        output_schema = descriptor["outputSchema"]["properties"]["value"]
        assert output_schema["properties"]["compound"]["format"] == "double"
        score = lambda text: mcp("tools/call", {"name": "analyze_sentiment", "arguments": {"turn": turn, "input": {"text": text}}})
        if mode != "no_tools":
            assert score(42)["isError"]  # Product input contract, before callback.
            for invalid in ["", " ", "invented excerpt"]:
                assert score(invalid)["isError"]  # Application validation.
            positive = "The new search is wonderful! I love how quickly I can find my notes."
            negative = "The export feature is frustrating and unreliable. I lost an afternoon trying to download my work."
            assert positive in prompt and negative in prompt
            for text, sign in [(positive, 1), (negative, -1)]:
                result = score(text)
                assert not result["isError"]
                scores = result["structuredContent"]["value"]
                assert scores["compound"] * sign > 0
                assert all(0 <= scores[k] <= 1 for k in ["positive", "negative", "neutral"])
        if mode != "missing_result":
            value = {"overall_tone": "mixed", "passages": [{"title": "Search", "summary": "Search is praised; export needs work."}], "recommendation": "Prioritize export reliability."}
            invalid = dict(value, overall_tone="invented")
            assert submit(invalid)["isError"]
            assert not submit(value)["isError"]
        record.write_text(str(turns))
        reply(identifier, {"stopReason": "end_turn"})

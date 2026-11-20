"""Deterministic probe checks; no installed agent or model calls required."""
import contextlib
import importlib.util
import io
import json
from pathlib import Path
import sys
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location(
    "probe_agent", Path(__file__).with_name("probe_agent.py")
)
probe = importlib.util.module_from_spec(spec)
spec.loader.exec_module(probe)

PEER = r'''
import json
import sys

def send(message):
    print(json.dumps(message), flush=True)

request = json.loads(sys.stdin.readline())
identifier = request["id"]
# A peer's request IDs are independent of the client's IDs.
send({"jsonrpc": "2.0", "id": identifier, "method": "fs/read_text_file",
      "params": {"path": "/unused"}})
reply = json.loads(sys.stdin.readline())
if reply.get("method"):
    # The buggy probe moves on to session/new instead of rejecting our request.
    # Let it finish cleanly so the test reports the missing initialization result.
    send({"jsonrpc": "2.0", "id": reply["id"], "error": {
        "code": -32600, "message": "still awaiting rejection of peer request"}})
    for line in sys.stdin:
        pass
    sys.exit(0)
assert reply["id"] == identifier and reply["error"]["code"] == -32601
send({"jsonrpc": "2.0", "id": identifier, "result": {
    "protocolVersion": 1, "agentInfo": {"name": "collision-fixture"}}})
request = json.loads(sys.stdin.readline())
assert request["method"] == "session/new"
send({"jsonrpc": "2.0", "id": request["id"], "result": {"sessionId": "fixture"}})
# Remain available until the probe performs its normal cleanup.
for line in sys.stdin:
    raise AssertionError("unexpected extra request")
'''


class ProbeTests(unittest.TestCase):
    def test_incoming_request_with_matching_id_is_rejected_before_response(self):
        output = io.StringIO()
        with patch.object(probe, "COMMAND", [sys.executable, "-u", "-c", PEER]):
            with contextlib.redirect_stdout(output):
                probe.main()
        result = json.loads(output.getvalue())
        self.assertNotIn("probe_error", result)
        self.assertEqual(result["initialize"].get("agentInfo"), {"name": "collision-fixture"})
        self.assertIn("session", result)
        self.assertEqual(result["prompts_sent"], 0)


if __name__ == "__main__":
    unittest.main()

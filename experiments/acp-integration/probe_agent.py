#!/usr/bin/env python3
"""Bounded capability probe; sends no prompts and prints no auth material."""
import json
import os
import selectors
import signal
import subprocess
import tempfile
import time

COMMAND = ["npx", "--offline", "--yes", "@zed-industries/claude-agent-acp@0.23.1"]


def main():
    result = {"command": COMMAND, "prompts_sent": 0}
    with tempfile.TemporaryDirectory(prefix="thinkwell-acp-probe-") as cwd:
        # Keep unsolicited stderr (potentially containing environment details) private.
        with tempfile.TemporaryFile() as errors:
            process = subprocess.Popen(
                COMMAND, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                stderr=errors, cwd=cwd, start_new_session=True,
            )
            selector = selectors.DefaultSelector()
            selector.register(process.stdout, selectors.EVENT_READ)
            buffer = b""

            def request(identifier, method, params):
                nonlocal buffer
                message = {"jsonrpc": "2.0", "id": identifier,
                           "method": method, "params": params}
                process.stdin.write((json.dumps(message) + "\n").encode())
                process.stdin.flush()
                deadline = time.monotonic() + 25
                while time.monotonic() < deadline:
                    if b"\n" not in buffer:
                        if not selector.select(max(0, deadline - time.monotonic())):
                            break
                        chunk = os.read(process.stdout.fileno(), 65536)
                        if not chunk:
                            raise RuntimeError("agent exited before response")
                        buffer += chunk
                        continue
                    line, buffer = buffer.split(b"\n", 1)
                    response = json.loads(line)
                    if response.get("id") == identifier:
                        return response
                    # No filesystem, terminal or permission request is approved by this probe.
                    if "id" in response and "method" in response:
                        process.stdin.write((json.dumps({"jsonrpc":"2.0", "id":response["id"],
                            "error":{"code":-32601, "message":"Capability probe only"}}) + "\n").encode())
                        process.stdin.flush()
                raise TimeoutError(f"timeout awaiting {method}")

            try:
                response = request(1, "initialize", {
                    "protocolVersion": 1,
                    "clientCapabilities": {},
                    "clientInfo": {"name": "thinkwell-capability-probe", "version": "0.0.0"},
                })
                init = response.get("result", {})
                result["initialize"] = {k: init[k] for k in
                    ("protocolVersion", "agentInfo", "agentCapabilities") if k in init}
                if "error" in response:
                    result["initialize_error_code"] = response["error"].get("code")
                else:
                    response = request(2, "session/new", {"cwd": cwd, "mcpServers": []})
                    session = response.get("result", {})
                    result["session"] = {k: session[k] for k in
                        ("configOptions", "models", "modes") if k in session}
                    if "error" in response:
                        result["session_error_code"] = response["error"].get("code")
                    options = session.get("configOptions", [])
                    model = next((x for x in options if x.get("category") == "model"), None)
                    if model:
                        response = request(3, "session/set_config_option", {
                            "sessionId": session["sessionId"], "configId": model["id"],
                            "value": model["currentValue"],
                        })
                        result["set_current_model"] = {
                            "success": "result" in response,
                            "error_code": response.get("error", {}).get("code"),
                        }
            except (TimeoutError, RuntimeError, BrokenPipeError, json.JSONDecodeError) as error:
                result["probe_error"] = str(error)
            finally:
                selector.close()
                try:
                    os.killpg(process.pid, signal.SIGTERM)
                except ProcessLookupError:
                    pass
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    os.killpg(process.pid, signal.SIGKILL)
                    process.wait(timeout=5)
                process.stdin.close()
                process.stdout.close()
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()

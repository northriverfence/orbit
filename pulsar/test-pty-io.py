#!/usr/bin/env python3
"""
Test script for Pulsar PTY I/O functionality

Tests the complete flow:
1. Connect to daemon via Unix socket
2. Create a session
3. Send input to PTY
4. Receive output from PTY
"""

import socket
import json
import base64
import sys
import time
import os

SOCKET_PATH = os.path.expanduser("~/.config/orbit/pulsar.sock")

class DaemonClient:
    def __init__(self, socket_path):
        self.socket_path = socket_path
        self.sock = None
        self.request_id = 0

    def connect(self):
        """Connect to the daemon Unix socket"""
        self.sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        try:
            self.sock.connect(self.socket_path)
            print(f"✓ Connected to daemon at {self.socket_path}")
            return True
        except Exception as e:
            print(f"✗ Failed to connect: {e}")
            return False

    def send_request(self, method, params):
        """Send JSON-RPC request and receive response"""
        self.request_id += 1
        request = {
            "id": str(self.request_id),
            "method": method,
            "params": params
        }

        # Send request
        request_json = json.dumps(request) + "\n"
        self.sock.sendall(request_json.encode())
        print(f"→ Sent: {method}")

        # Receive response
        response_data = b""
        while b"\n" not in response_data:
            chunk = self.sock.recv(4096)
            if not chunk:
                raise Exception("Connection closed by daemon")
            response_data += chunk

        response_json = response_data.decode().strip()
        response = json.loads(response_json)
        print(f"← Received response for: {method}")

        # Check for errors
        if "error" in response:
            error = response["error"]
            raise Exception(f"Daemon error {error['code']}: {error['message']}")

        return response.get("result", {})

    def create_session(self, name, session_type="Local"):
        """Create a new terminal session"""
        params = {
            "name": name,
            "type": session_type
        }
        result = self.send_request("create_session", params)
        session_id = result.get("session_id")
        print(f"✓ Created session: {session_id}")
        return session_id

    def send_input(self, session_id, data):
        """Send input to PTY (data should be string, will be base64-encoded)"""
        data_base64 = base64.b64encode(data.encode()).decode()
        params = {
            "session_id": session_id,
            "data": data_base64
        }
        result = self.send_request("send_input", params)
        bytes_written = result.get("bytes_written", 0)
        print(f"✓ Sent {bytes_written} bytes to PTY")
        return bytes_written

    def receive_output(self, session_id, timeout_ms=None):
        """Receive output from PTY"""
        params = {
            "session_id": session_id
        }
        if timeout_ms is not None:
            params["timeout_ms"] = timeout_ms

        result = self.send_request("receive_output", params)
        data_base64 = result.get("data", "")
        bytes_read = result.get("bytes_read", 0)

        if bytes_read > 0:
            data = base64.b64decode(data_base64).decode('utf-8', errors='replace')
            print(f"✓ Received {bytes_read} bytes from PTY")
            return data
        else:
            print("✓ No output available")
            return ""

    def list_sessions(self):
        """List all sessions"""
        result = self.send_request("list_sessions", {})
        sessions = result.get("sessions", [])
        print(f"✓ Found {len(sessions)} session(s)")
        return sessions

    def get_status(self):
        """Get daemon status"""
        result = self.send_request("get_status", {})
        print(f"✓ Daemon status: {result}")
        return result

    def terminate_session(self, session_id):
        """Terminate a session"""
        params = {"session_id": session_id}
        self.send_request("terminate_session", params)
        print(f"✓ Terminated session: {session_id}")

    def close(self):
        """Close the socket connection"""
        if self.sock:
            self.sock.close()
            print("✓ Disconnected from daemon")


def test_pty_io():
    """Run end-to-end PTY I/O test"""
    print("=" * 60)
    print("Pulsar PTY I/O End-to-End Test")
    print("=" * 60)
    print()

    client = DaemonClient(SOCKET_PATH)

    try:
        # Step 1: Connect
        print("Step 1: Connecting to daemon...")
        if not client.connect():
            print("✗ Cannot connect. Is the daemon running?")
            print(f"  Run: cd pulsar-daemon && cargo run")
            return False
        print()

        # Step 2: Get status
        print("Step 2: Getting daemon status...")
        status = client.get_status()
        print(f"  Version: {status.get('version')}")
        print(f"  Uptime: {status.get('uptime_seconds')}s")
        print(f"  Sessions: {status.get('num_sessions')}")
        print()

        # Step 3: Create session
        print("Step 3: Creating terminal session...")
        session_id = client.create_session("test-session")
        print()

        # Step 4: Send simple command
        print("Step 4: Sending command 'echo Hello Pulsar'...")
        client.send_input(session_id, "echo Hello Pulsar\n")
        print()

        # Step 5: Wait for output
        print("Step 5: Waiting for output...")
        time.sleep(0.5)  # Give shell time to execute
        print()

        # Step 6: Receive output
        print("Step 6: Receiving output...")
        output = client.receive_output(session_id)
        if output:
            print("Output:")
            print("-" * 60)
            print(output)
            print("-" * 60)
        print()

        # Step 7: Send another command
        print("Step 7: Sending command 'pwd'...")
        client.send_input(session_id, "pwd\n")
        time.sleep(0.3)
        print()

        # Step 8: Receive output again
        print("Step 8: Receiving output...")
        output = client.receive_output(session_id)
        if output:
            print("Output:")
            print("-" * 60)
            print(output)
            print("-" * 60)
        print()

        # Step 9: List sessions
        print("Step 9: Listing sessions...")
        sessions = client.list_sessions()
        for sess in sessions:
            print(f"  - {sess['name']} ({sess['id']})")
            print(f"    State: {sess['state']}, Clients: {sess['num_clients']}")
        print()

        # Step 10: Terminate session
        print("Step 10: Terminating session...")
        client.terminate_session(session_id)
        print()

        # Step 11: Verify termination
        print("Step 11: Verifying session terminated...")
        sessions = client.list_sessions()
        print(f"  Active sessions: {len(sessions)}")
        print()

        print("=" * 60)
        print("✓ All tests passed!")
        print("=" * 60)
        return True

    except Exception as e:
        print()
        print("=" * 60)
        print(f"✗ Test failed: {e}")
        print("=" * 60)
        import traceback
        traceback.print_exc()
        return False

    finally:
        client.close()


if __name__ == "__main__":
    success = test_pty_io()
    sys.exit(0 if success else 1)

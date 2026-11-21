"""Linux PTY regression tests. Standard-library only; never invokes a host shell."""
import fcntl
import os
import pty
import re
import select
import signal
import struct
import subprocess
import sys
import termios
import time

PROMPT = b"visitor@gino.os:~$ "
ANSI = re.compile(rb"\x1b\[[0-?]*[ -/]*[@-~]")


class Terminal:
    def __init__(self):
        self.master, self.slave = pty.openpty()
        fcntl.ioctl(self.slave, termios.TIOCSWINSZ, struct.pack("HHHH", 24, 80, 0, 0))
        self.before = termios.tcgetattr(self.slave)
        self.buffer = b""
        env = dict(os.environ, TERM="xterm-256color")
        self.process = subprocess.Popen(
            [sys.argv[1]], stdin=self.slave, stdout=self.slave, stderr=self.slave,
            env=env, start_new_session=True,
        )
        self.until(PROMPT)

    def until(self, marker):
        end = time.monotonic() + 5
        while marker not in self.buffer:
            remaining = end - time.monotonic()
            assert remaining > 0, (marker, self.buffer)
            if select.select([self.master], [], [], remaining)[0]:
                data = os.read(self.master, 65536)
                assert data, self.buffer
                self.buffer += data
                if b"\x1b[6n" in data:
                    os.write(self.master, b"\x1b[1;1R")
        position = self.buffer.index(marker) + len(marker)
        result, self.buffer = self.buffer[:position], self.buffer[position:]
        return result

    def command(self, text):
        os.write(self.master, text + b"\r")
        self.until(b"\x1b[?2004l")
        return self.until(PROMPT)

    def finish(self, data, status):
        os.write(self.master, data)
        assert self.process.wait(timeout=5) == status
        assert termios.tcgetattr(self.slave) == self.before, "terminal settings not restored"

    def close(self):
        if self.process.poll() is None:
            self.process.kill()
            self.process.wait(timeout=5)
        os.close(self.master)
        os.close(self.slave)


def output_has(output, expected):
    assert expected in ANSI.sub(b"", output), output


t = Terminal()
try:
    output_has(t.command(b"help"), b"Available commands:")
    output_has(t.command(b"pwd"), b"/home/visitor\r\n")
    output_has(t.command(b"whoami"), b"visitor\r\n")
    output_has(t.command('echo "hello 🦀"'.encode()), 'hello 🦀\r\n'.encode())
    output_has(t.command(b"echo history-marker"), b"history-marker\r\n")
    output_has(t.command(b"\x1b[A"), b"history-marker\r\n")
    # Edit in the middle and remove the unwanted final character.
    output_has(t.command(b"echo ac\x1b[Db\x1b[Fz\x7f"), b"abc\r\n")
    output_has(t.command(b"echo before | whoami"), b"not supported yet")
    output_has(t.command(b"/bin/sh"), b"command not found")
    output_has(t.command(b"\x1b[200~echo first\necho second\x1b[201~"), b"not supported yet")
    assert b"\x1b[2J\x1b[H" in t.command(b"clear")
    # Ctrl+C discards the input, keeps the shell alive and resets the prompt.
    os.write(t.master, b"echo SHOULD_NOT_RUN\x03")
    t.until(b"\x1b[?2004l")
    output_has(t.until(PROMPT), b"^C")
    output_has(t.command(b"echo recovered"), b"recovered\r\n")
    # Ctrl+D on nonempty input deletes at the cursor, not the session.
    output_has(t.command(b"echo ac\x1b[D\x04b"), b"ab\r\n")
    # Resize the terminal during editing.
    fcntl.ioctl(t.slave, termios.TIOCSWINSZ, struct.pack("HHHH", 30, 100, 0, 0))
    os.kill(t.process.pid, signal.SIGWINCH)
    output_has(t.command(b"echo resized"), b"resized\r\n")
    os.kill(t.process.pid, signal.SIGINT)
    t.until(b"\x1b[?2004l")
    output_has(t.until(PROMPT), b"^C")
    output_has(t.command(b"echo signal-recovered"), b"signal-recovered\r\n")
    t.finish(b"exit 7\r", 7)
finally:
    t.close()

# EOF returns the last shell status and restores termios after a syntax failure.
t = Terminal()
try:
    output_has(t.command(b"echo 'broken"), b"unclosed")
    t.finish(b"\x04", 2)
finally:
    t.close()

# Ctrl+C status is visible to exit without an explicit status.
t = Terminal()
try:
    os.write(t.master, b"\x03")
    t.until(b"\x1b[?2004l")
    t.until(PROMPT)
    t.finish(b"exit\r", 130)
finally:
    t.close()
# History is bounded and is not loaded from previous processes.
t = Terminal()
try:
    for index in range(105):
        output_has(t.command(f"echo history-{index}".encode()), f"history-{index}\r\n".encode())
    output_has(t.command(b"\x1b[A" * 110), b"history-5\r\n")
    t.finish(b"\x04", 0)
finally:
    t.close()
print("PTY checks passed: commands, editing, history, Ctrl+C/D, resize, exit status and termios restoration")

# PR-09: the identical reviewed scenario also runs in the production browser.
# A PTY combines stdout/stderr; engine parity tests check their distinction.
import json
from pathlib import Path
fixtures = json.loads((Path(__file__).resolve().parents[3] / "tests/parity/fixtures.json").read_text())
for scenario in (fixture for fixture in fixtures if fixture.get("smoke")):
    t = Terminal()
    try:
        for step in scenario["steps"]:
            expected = step["expected"]
            if expected["session"]["exitStatus"] is not None:
                t.finish(step["input"].encode() + b"\r", expected["status"])
                break
            output = t.command(step["input"].encode())
            if any(event["type"] == "clearScreen" for event in expected["events"]):
                assert b"\x1b[2J\x1b[H" in output, output
            else:
                actual = ANSI.sub(b"", output).removesuffix(PROMPT).replace(b"\r", b"").strip(b"\n")
                text = "".join(event["text"] for event in expected["events"]).encode().strip(b"\n")
                assert actual == text, (scenario["name"], step["input"], actual, text)
    finally:
        t.close()
print("Shared parity fixture smoke passed through native CLI PTY")

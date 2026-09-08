"""Exercise one owned controls-app native window through its real AT-SPI tree."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import sys
import time

from dbus import DBusException

from atspi_client import Client, host_bus, ppm_checksum, verify_controls, verify_label


def eventually(description, operation, timeout=15):
    deadline = time.monotonic() + timeout
    failure = None
    while time.monotonic() < deadline:
        try:
            result = operation()
            if result is not None:
                return result
        except (OSError, ValueError, KeyError, StopIteration, DBusException) as error:
            failure = error
        time.sleep(0.05)
    raise RuntimeError(f"timeout waiting for {description}: {failure}")


def selected(nodes, name):
    matches = [node for node in nodes if node["accessible_id"] == name]
    if len(matches) != 1:
        raise ValueError(f"expected unique accessible control {name}")
    return matches[0]


def control(state, name):
    return next(item for item in state["controls"] if item["name"] == name)


class Probe:
    def __init__(self, client, root, pid, output):
        self.client = client
        self.root = root
        self.pid = pid
        self.output = output
        self.checkpoints = []
        self.identities = None

    def observed(self):
        state = json.loads((self.output / "state.json").read_text())
        if state["pid"] != self.pid or state["presentations"] < 1:
            raise ValueError("export must belong to the presented owned process")
        frame = (self.output / state["frame"]).resolve()
        if not frame.is_relative_to(self.output.resolve()):
            raise ValueError("frame export escaped the output directory")
        if ppm_checksum(frame.read_bytes()) != state["rgba_checksum"]:
            raise ValueError("presented PPM does not match the Rust RGBA checksum")
        nodes = self.client.tree(self.root)
        # Preserve the latest raw observation even when a semantic assertion fails.
        (self.output / "latest-observation.json").write_text(json.dumps(
            {"rust": state, "atspi": nodes}, indent=2) + "\n")
        return state, nodes, frame

    def checkpoint(self, name, predicate):
        def accepted():
            state, nodes, frame = self.observed()
            if not predicate(state):
                return None
            verify_controls(nodes, state["controls"])
            verify_label(nodes, "readout", state["readout"])
            identities = {
                item["name"]: selected(nodes, item["name"])["path"]
                for item in state["controls"]
            }
            if self.identities is not None and self.identities != identities:
                raise ValueError("control object paths changed after an action")
            self.identities = identities
            return state, nodes, frame

        state, nodes, frame = eventually(name, accepted)
        for item in state["controls"]:
            if selected(nodes, item["name"])["children"]:
                raise ValueError("composed control text leaked as duplicate accessible children")
        shutil.copyfile(frame, self.output / f"{name}.ppm")
        record = {"stage": name, "rust": state, "atspi": nodes,
                  "ppm_sha256": hashlib.sha256(frame.read_bytes()).hexdigest()}
        (self.output / f"{name}.json").write_text(json.dumps(record, indent=2) + "\n")
        self.checkpoints.append(record)
        print(f"{name}: generation={state['generation']} checksum={state['rgba_checksum']}",
              flush=True)
        return state, nodes

    def action(self, nodes, name, action):
        if not self.client.act(selected(nodes, name), action):
            raise ValueError(f"AT-SPI rejected {action} on {name}")

    def exercise(self):
        initial, nodes = self.checkpoint("initial", lambda state:
            not control(state, "compact")["checked"]
            and control(state, "apply")["disabled"] and state["apply_count"] == 0)
        self.action(nodes, "compact", "focus")
        focused, nodes = self.checkpoint("focused", lambda state:
            control(state, "compact")["focused"])
        self.action(nodes, "compact", "click")
        changed, nodes = self.checkpoint("changed", lambda state:
            control(state, "compact")["checked"]
            and not control(state, "apply")["disabled"] and state["apply_count"] == 0)
        self.action(nodes, "apply", "click")
        applied, nodes = self.checkpoint("applied", lambda state:
            state["apply_count"] == 1 and control(state, "apply")["disabled"])
        if len({state["rgba_checksum"] for state in (initial, focused, changed, applied)}) != 4:
            raise ValueError("focus, check and apply must each change presented pixels")
        if not control(applied, "compact")["focused"]:
            raise ValueError("AT-SPI activation unexpectedly moved logical focus")
        self.action(nodes, "finish", "click")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--inspect-only", action="store_true")
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    command = args.command[1:] if args.command[:1] == ["--"] else args.command
    if not command:
        parser.error("supply the built a11y-probe executable and its output directory")
    args.output.mkdir(parents=True, exist_ok=True)
    if any(args.output.iterdir()):
        parser.error("use an empty output directory to exclude stale or unrelated files")
    address, before = host_bus()
    log = (args.output / "native.log").open("w")
    fixture = Path(__file__).with_name("private_session.py")
    process = subprocess.Popen([
        "dbus-run-session", "--", sys.executable, str(fixture), "--address", address,
        "--pid-file", str(args.output / "child.json"), "--", *command,
    ], stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
    probe = None
    failure = None
    try:
        child = eventually("owned child PID", lambda:
            json.loads((args.output / "child.json").read_text()))
        client = Client(address)
        root = eventually("real AT-SPI application", lambda:
            client.application_for_pid(child["child_pid"]))
        probe = Probe(client, root, child["child_pid"], args.output)
        if args.inspect_only:
            _, nodes, _ = eventually("presented observation", probe.observed)
            probe.action(nodes, "finish", "click")
        else:
            probe.exercise()
        if process.wait(timeout=10) != 0:
            raise RuntimeError("native probe did not exit successfully")
    except Exception as error:
        failure = str(error)
    finally:
        if process.poll() is None:
            # This group belongs only to the wrapper, fixture and child we created.
            try:
                os.killpg(process.pid, signal.SIGTERM)
            except ProcessLookupError:
                pass
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                try:
                    os.killpg(process.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
                process.wait()
        log.close()
        _, after = host_bus()
        if before != after:
            failure = f"host accessibility status changed: {before} -> {after}"
        report = {
            "activation": "private session Status fixture; real host AT-SPI tree and actions",
            "host_status_before": before, "host_status_after": after,
            "inspect_only": args.inspect_only, "success": failure is None,
            "failure": failure, "checkpoints": probe.checkpoints if probe else [],
        }
        (args.output / "report.json").write_text(json.dumps(report, indent=2) + "\n")
    if failure:
        raise RuntimeError(f"{failure}; see {args.output / 'report.json'} and native.log")
    print(f"AT-SPI probe passed; evidence: {args.output / 'report.json'}")


if __name__ == "__main__":
    main()

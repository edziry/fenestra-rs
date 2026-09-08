"""Process-local activation fixture; the application still uses the real AT-SPI bus."""

import argparse
import json
import os
from pathlib import Path
import signal
import subprocess
import sys

import dbus
import dbus.mainloop.glib
import dbus.service
from gi.repository import GLib


class Status(dbus.service.Object):
    def __init__(self, connection, address):
        super().__init__(connection, "/org/a11y/bus")
        self.address = address

    @dbus.service.method("org.freedesktop.DBus.Properties", in_signature="ss",
                         out_signature="v")
    def Get(self, interface, name):
        return self.GetAll(interface)[name]

    @dbus.service.method("org.freedesktop.DBus.Properties", in_signature="s",
                         out_signature="a{sv}")
    def GetAll(self, interface):
        if interface != "org.a11y.Status":
            raise dbus.exceptions.DBusException("unknown fixture interface")
        return {"IsEnabled": dbus.Boolean(True), "ScreenReaderEnabled": dbus.Boolean(False)}

    @dbus.service.method("org.a11y.Bus", out_signature="s")
    def GetAddress(self):
        return self.address


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--address", required=True)
    parser.add_argument("--pid-file", type=Path, required=True)
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    command = args.command[1:] if args.command[:1] == ["--"] else args.command
    if not command:
        parser.error("a child command is required")
    dbus.mainloop.glib.DBusGMainLoop(set_as_default=True)
    connection = dbus.SessionBus()
    connection.request_name("org.a11y.Bus", dbus.bus.NAME_FLAG_DO_NOT_QUEUE)
    status = Status(connection, args.address)
    child = subprocess.Popen(command, env={**os.environ, "AT_SPI_BUS_ADDRESS": args.address})
    args.pid_file.write_text(json.dumps({"child_pid": child.pid, "fixture_pid": os.getpid()}))
    loop = GLib.MainLoop()

    def finished():
        if child.poll() is None:
            return True
        loop.quit()
        return False

    def interrupted(_signal, _frame):
        child.terminate()
        loop.quit()

    signal.signal(signal.SIGTERM, interrupted)
    signal.signal(signal.SIGINT, interrupted)
    GLib.timeout_add(100, finished)
    try:
        loop.run()
    finally:
        if child.poll() is None:
            child.terminate()
            try:
                child.wait(timeout=3)
            except subprocess.TimeoutExpired:
                child.kill()
        status.remove_from_connection()
        if connection.get_is_connected():
            try:
                connection.release_name("org.a11y.Bus")
            except dbus.DBusException:
                pass
    return child.wait()


if __name__ == "__main__":
    sys.exit(main())

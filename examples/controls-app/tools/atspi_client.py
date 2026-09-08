"""Small, uncached AT-SPI D-Bus client for the owned native probe process."""

from collections import deque


FLAGS = {
    "checked": 4, "enabled": 8, "focusable": 11, "focused": 12,
    "pressed": 20, "sensitive": 24, "showing": 25, "visible": 30,
    "checkable": 41,
}
ACCESSIBLE = "org.a11y.atspi.Accessible"
COMPONENT = "org.a11y.atspi.Component"
ACTION = "org.a11y.atspi.Action"
PROPERTIES = "org.freedesktop.DBus.Properties"
ROOT = ("org.a11y.atspi.Registry", "/org/a11y/atspi/accessible/root")
# AT-SPI Role enum values; unknown roles retain their numeric identity in reports.
ROLES = {7: "check box", 23: "frame", 29: "label", 39: "panel",
         43: "push button", 75: "application"}


def decode_states(words):
    return {
        name: bit // 32 < len(words) and bool(words[bit // 32] & (1 << (bit % 32)))
        for name, bit in FLAGS.items()
    }


def inspect_tree(root, read_node, max_nodes=128):
    pending = deque([root])
    seen = set()
    result = []
    while pending:
        reference = pending.popleft()
        if reference in seen:
            raise ValueError("accessible tree cycle or duplicate child")
        if len(seen) >= max_nodes:
            raise ValueError("accessible tree node budget exceeded")
        seen.add(reference)
        node = read_node(reference)
        result.append(node)
        pending.extend(node["children"])
    return result


def verify_controls(nodes, expected):
    roles = {"button": "push button", "checkbox": "check box"}
    for control in expected:
        name = control["name"]
        matches = [node for node in nodes if node["accessible_id"] == name]
        if len(matches) != 1:
            raise ValueError(f"{name}: expected one unique AccessibleId, got {len(matches)}")
        node = matches[0]
        checks = {
            "role": (node["role"], roles[control["role"]]),
            "label": (node["name"], control["label"]),
            "bounds": (node["bounds"], control["bounds"]),
            "enabled": (node["states"]["enabled"], not control["disabled"]),
            "sensitive": (node["states"]["sensitive"], not control["disabled"]),
            "focused": (node["states"]["focused"], control["focused"]),
        }
        if control["checked"] is not None:
            checks["checked"] = (node["states"]["checked"], control["checked"])
        for field, (actual, wanted) in checks.items():
            if actual != wanted:
                raise ValueError(f"{name}: {field} OS={actual!r}, Rust={wanted!r}")


def ppm_checksum(data):
    magic, dimensions, maximum, pixels = data.split(b"\n", 3)
    if magic != b"P6" or maximum != b"255":
        raise ValueError("expected the probe's binary PPM header")
    width, height = map(int, dimensions.split())
    if width < 1 or height < 1 or len(pixels) != width * height * 3:
        raise ValueError("PPM pixel length mismatch")
    value = 0xCBF29CE484222325
    for index, byte in enumerate(pixels):
        value = ((value ^ byte) * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
        if index % 3 == 2:
            value = ((value ^ 255) * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return f"{value:016x}"


class Client:
    def __init__(self, address):
        import dbus

        self.dbus = dbus
        self.bus = dbus.bus.BusConnection(address)

    def interface(self, reference, interface):
        name, path = reference
        return self.dbus.Interface(
            self.bus.get_object(name, path, introspect=False), interface
        )

    def children(self, reference):
        return [
            (str(name), str(path))
            for name, path in self.interface(reference, ACCESSIBLE).GetChildren(timeout=2)
        ]

    def application_for_pid(self, pid):
        daemon = self.interface(("org.freedesktop.DBus", "/org/freedesktop/DBus"),
                                "org.freedesktop.DBus")
        for reference in self.children(ROOT):
            try:
                owner_pid = int(daemon.GetConnectionUnixProcessID(reference[0], timeout=2))
            except self.dbus.DBusException:
                continue
            if owner_pid == pid:
                return reference
        raise ValueError(f"owned PID {pid} has not registered an AT-SPI application")

    def read_node(self, reference):
        accessible = self.interface(reference, ACCESSIBLE)
        properties = self.interface(reference, PROPERTIES)
        interfaces = list(map(str, accessible.GetInterfaces(timeout=2)))
        role = int(accessible.GetRole(timeout=2))
        node = {
            "bus": reference[0], "path": reference[1],
            "name": str(properties.Get(ACCESSIBLE, "Name", timeout=2)),
            "accessible_id": str(properties.Get(ACCESSIBLE, "AccessibleId", timeout=2)),
            "role_id": role, "role": ROLES.get(role, f"role {role}"),
            "states": decode_states(accessible.GetState(timeout=2)),
            "children": self.children(reference), "interfaces": interfaces,
            "bounds": None, "actions": [],
        }
        if COMPONENT in interfaces:
            node["bounds"] = list(map(int, self.interface(reference, COMPONENT)
                                      .GetExtents(self.dbus.UInt32(1), timeout=2)))
        if ACTION in interfaces:
            action = self.interface(reference, ACTION)
            count = int(properties.Get(ACTION, "NActions", timeout=2))
            node["actions"] = [str(action.GetName(self.dbus.Int32(i), timeout=2))
                               for i in range(count)]
        return node

    def tree(self, root):
        def read_owned(reference):
            if reference[0] != root[0]:
                raise ValueError("refusing to inspect a foreign application's child")
            return self.read_node(reference)

        return inspect_tree(root, read_owned)

    def act(self, node, action):
        reference = (node["bus"], node["path"])
        if action == "focus":
            return bool(self.interface(reference, COMPONENT).GrabFocus(timeout=2))
        if action == "click":
            index = node["actions"].index("click")
            return bool(self.interface(reference, ACTION)
                        .DoAction(self.dbus.Int32(index), timeout=2))
        raise ValueError(f"unsupported probe action {action}")


def host_bus():
    import dbus

    session = dbus.SessionBus()
    service = session.get_object("org.a11y.Bus", "/org/a11y/bus", introspect=False)
    properties = dbus.Interface(service, PROPERTIES)
    status = {
        key: bool(properties.Get("org.a11y.Status", key, timeout=2))
        for key in ("IsEnabled", "ScreenReaderEnabled")
    }
    address = str(dbus.Interface(service, "org.a11y.Bus").GetAddress(timeout=2))
    return address, status

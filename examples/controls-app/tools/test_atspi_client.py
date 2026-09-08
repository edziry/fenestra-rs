import unittest
from types import SimpleNamespace

from atspi_client import ACTION, COMPONENT, Client, decode_states, inspect_tree, verify_controls, ppm_checksum


class ProbeTests(unittest.TestCase):
    def test_state_words_include_flags_above_32(self):
        states = decode_states([(1 << 4) | (1 << 8), 1 << 9])
        self.assertTrue(states["checked"])
        self.assertTrue(states["enabled"])
        self.assertTrue(states["checkable"])
        self.assertFalse(states["focused"])

    def test_traversal_keeps_identity_and_rejects_cycles(self):
        nodes = {"root": {"children": ["child"]}, "child": {"children": []}}
        self.assertEqual(len(inspect_tree("root", nodes.__getitem__)), 2)
        nodes["child"]["children"] = ["root"]
        with self.assertRaisesRegex(ValueError, "cycle"):
            inspect_tree("root", nodes.__getitem__)

    def test_traversal_enforces_node_budget(self):
        nodes = {"root": {"children": ["child"]}, "child": {"children": []}}
        with self.assertRaisesRegex(ValueError, "budget"):
            inspect_tree("root", nodes.__getitem__, max_nodes=1)

    def test_tree_never_reads_a_foreign_application(self):
        client = object.__new__(Client)
        reads = []

        def read(reference):
            reads.append(reference)
            return {"children": [(":foreign", "/child")]}

        client.read_node = read
        with self.assertRaisesRegex(ValueError, "foreign"):
            client.tree((":owned", "/root"))
        self.assertEqual(reads, [(":owned", "/root")])

    def test_role_query_uses_numeric_contract_without_get_role_name(self):
        class Accessible:
            def GetInterfaces(self, **_):
                return []

            def GetRole(self, **_):
                return 43

            def GetState(self, **_):
                return []

            def Get(self, _interface, name, **_):
                return {"Name": "Apply", "AccessibleId": "apply"}[name]

        client = object.__new__(Client)
        client.interface = lambda *_: Accessible()
        client.children = lambda *_: []
        node = client.read_node((":owned", "/apply"))
        self.assertEqual(node["role_id"], 43)
        self.assertEqual(node["role"], "push button")

    def test_dbus_integer_arguments_use_exact_wire_signatures(self):
        class UInt32(int):
            pass

        class Int32(int):
            pass

        test = self

        class Accessible:
            def GetInterfaces(self, **_):
                return [COMPONENT, ACTION]

            def GetRole(self, **_):
                return 43

            def GetState(self, **_):
                return []

            def Get(self, _interface, name, **_):
                return {"Name": "Apply", "AccessibleId": "apply", "NActions": 1}[name]

            def GetExtents(self, coordinate, **_):
                test.assertIsInstance(coordinate, UInt32)
                return [1, 2, 3, 4]

            def GetName(self, index, **_):
                test.assertIsInstance(index, Int32)
                return "click"

            def DoAction(self, index, **_):
                test.assertIsInstance(index, Int32)
                return True

        client = object.__new__(Client)
        client.dbus = SimpleNamespace(UInt32=UInt32, Int32=Int32)
        client.interface = lambda *_: Accessible()
        client.children = lambda *_: []
        node = client.read_node((":owned", "/apply"))
        self.assertEqual(node["bounds"], [1, 2, 3, 4])
        self.assertTrue(client.act(node, "click"))

    def test_checks_disabled_semantics_and_exact_bounds(self):
        node = {
            "accessible_id": "apply", "role": "push button", "name": "Apply",
            "bounds": [2, 3, 40, 20], "states": {
                "enabled": False, "sensitive": False, "focused": False,
                "checked": False,
            },
        }
        expected = [{
            "name": "apply", "role": "button", "label": "Apply",
            "bounds": [2, 3, 40, 20], "disabled": True,
            "checked": None, "focused": False,
        }]
        verify_controls([node], expected)
        node["states"]["enabled"] = True
        with self.assertRaisesRegex(ValueError, "apply: enabled"):
            verify_controls([node], expected)
        node["states"]["enabled"] = False
        node["bounds"][0] = 9
        with self.assertRaisesRegex(ValueError, "apply: bounds"):
            verify_controls([node], expected)

    def test_checks_duplicate_or_missing_control_identity(self):
        with self.assertRaisesRegex(ValueError, "unique"):
            verify_controls([], [{"name": "compact"}])

    def test_ppm_checksum_reconstructs_opaque_rgba_and_rejects_truncation(self):
        # FNV-1a of RGBA [0, 0, 0, 255].
        self.assertEqual(ppm_checksum(b"P6\n1 1\n255\n\0\0\0"), "4d25077f9dcd5758")
        with self.assertRaisesRegex(ValueError, "length"):
            ppm_checksum(b"P6\n1 1\n255\n\0\0")


if __name__ == "__main__":
    unittest.main()

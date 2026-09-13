"""Runtime contract tests for bindings emitted by `cargo test --all-features`."""

from __future__ import annotations

import collections
import dataclasses
import decimal
import ipaddress
import pathlib
import sys
import typing
import unittest

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1]))

from bindings.AuditEvent import (  # noqa: E402
    AuditEvent,
    AuditEventCreated,
    AuditEventMutable,
)
from bindings.auth.Event import EventLogin, EventLogout  # noqa: E402
from bindings.Choice import Choice, ChoiceMany, ChoiceOne  # noqa: E402
from bindings.CycleA import CycleA  # noqa: E402
from bindings.Event import Event, EventCreated, EventDeleted, EventMoved  # noqa: E402
from bindings.ImmutableUser import ImmutableUser  # noqa: E402
from bindings.Item import Item  # noqa: E402
from bindings.JsonPayload import JsonPayload  # noqa: E402
from bindings.Left import Left  # noqa: E402
from bindings.models.Address import Address  # noqa: E402
from bindings.Nested import Nested  # noqa: E402
from bindings.Page import Page  # noqa: E402
from bindings.Pair import Pair  # noqa: E402
from bindings.Right import Right  # noqa: E402
from bindings.Root import Root  # noqa: E402
from bindings.Status import Status  # noqa: E402
from bindings.TypeCoverage import TypeCoverage  # noqa: E402
from bindings.User import User  # noqa: E402


class GeneratedBindingsTest(unittest.TestCase):
    def test_dataclass_options_and_fields(self) -> None:
        model = ImmutableUser(id=1, name="Ada")
        self.assertTrue(dataclasses.is_dataclass(model))
        self.assertEqual([f.name for f in dataclasses.fields(model)], ["id", "name"])
        self.assertFalse(hasattr(model, "__dict__"))
        with self.assertRaises(TypeError):
            ImmutableUser(1, "Ada")
        with self.assertRaises(dataclasses.FrozenInstanceError):
            model.name = "Grace"
        self.assertEqual(typing.get_type_hints(User)["address"], Address | None)
        self.assertEqual(typing.get_type_hints(User)["status"], Status)

    def test_enum_shapes_have_no_synthetic_tag(self) -> None:
        self.assertEqual(Status.Active.value, "Active")
        self.assertEqual(Status.OnHold.value, "on-hold")
        self.assertEqual(typing.get_args(Event), (EventCreated, EventMoved, EventDeleted))
        self.assertEqual([f.name for f in dataclasses.fields(EventLogin)], ["user_id"])
        self.assertEqual([f.name for f in dataclasses.fields(EventLogout)], ["user_id"])
        self.assertIsInstance(EventLogin(user_id=1), EventLogin)
        self.assertNotIsInstance(EventLogin(user_id=1), EventLogout)
        self.assertEqual(
            typing.get_args(AuditEvent), (AuditEventCreated, AuditEventMutable)
        )
        self.assertTrue(AuditEventCreated.__dataclass_params__.frozen)
        self.assertFalse(AuditEventMutable.__dataclass_params__.frozen)

    def test_cyclic_annotations_resolve_after_import(self) -> None:
        self.assertEqual(typing.get_type_hints(Left)["right"], Right | None)
        self.assertEqual(typing.get_type_hints(Right)["left"], Left | None)
        self.assertEqual(typing.get_type_hints(CycleA)["b"].__args__[0].__name__, "CycleB")

    def test_generic_annotations_remain_parameterized(self) -> None:
        root = typing.get_type_hints(Root)
        self.assertEqual(root["page"], Page[Item])
        self.assertEqual(root["paths"], Page[pathlib.Path])
        self.assertEqual(root["pair"], Pair[pathlib.Path, Item])
        self.assertEqual(root["choice"], Choice[Item])
        self.assertEqual(Choice[int], ChoiceOne[int] | ChoiceMany[int])
        page = typing.get_type_hints(Page)
        self.assertEqual(page["value"], Page.__parameters__[0])
        self.assertEqual(page["items"], list[Page.__parameters__[0] | None])

    def test_native_types_and_manual_overrides(self) -> None:
        hints = typing.get_type_hints(TypeCoverage)
        self.assertEqual(hints["queue"], collections.deque[Nested])
        self.assertEqual(hints["path"], pathlib.Path)
        self.assertEqual(hints["ip"], ipaddress.IPv4Address | ipaddress.IPv6Address)
        self.assertEqual(hints["cow"], str)
        self.assertEqual(hints["cow_slice"], list[int])
        self.assertEqual(hints["cow_nested"], list[Nested])
        self.assertEqual(hints["heap"], list[int])
        self.assertEqual(hints["amount"], decimal.Decimal)

    def test_json_alias_is_recursive(self) -> None:
        hints = typing.get_type_hints(JsonPayload)
        self.assertEqual(typing.get_origin(hints["map"]), dict)
        self.assertEqual(typing.get_args(hints["map"])[0], str)
        value = hints["value"]
        self.assertIn(str, typing.get_args(value))
        self.assertTrue(any(typing.get_origin(item) is list for item in typing.get_args(value)))
        self.assertTrue(any(typing.get_origin(item) is dict for item in typing.get_args(value)))


if __name__ == "__main__":
    unittest.main()

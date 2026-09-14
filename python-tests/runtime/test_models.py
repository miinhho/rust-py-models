from __future__ import annotations

import dataclasses
import typing
import unittest

from binding.AuditEvent import AuditEvent, AuditEventCreated, AuditEventMutable
from binding.auth.Event import EventLogin, EventLogout
from binding.Both import Both
from binding.DocumentedEvent import DocumentedEventCreated
from binding.DocumentedStatus import DocumentedStatus
from binding.Event import Event, EventCreated, EventDeleted, EventMoved
from binding.ImmutableUser import ImmutableUser
from binding.LegacyModel import LegacyModel
from binding.shared.Model import First, Second
from binding.Status import Status


class ModelContracts(unittest.TestCase):
    def test_dataclass_options_and_variant_overrides(self) -> None:
        model = ImmutableUser(id=1, name="Ada")
        self.assertTrue(dataclasses.is_dataclass(model))
        self.assertEqual(
            [field.name for field in dataclasses.fields(model)], ["id", "name"]
        )
        self.assertFalse(hasattr(model, "__dict__"))
        with self.assertRaises(TypeError):
            ImmutableUser(1, "Ada")
        with self.assertRaises(dataclasses.FrozenInstanceError):
            model.name = "Grace"

        created = AuditEventCreated(id=1)
        self.assertTrue(created.__dataclass_params__.frozen)
        self.assertFalse(hasattr(created, "__dict__"))
        with self.assertRaises(TypeError):
            AuditEventCreated(1)

        mutable = AuditEventMutable("before")
        self.assertFalse(mutable.__dataclass_params__.frozen)
        self.assertTrue(hasattr(mutable, "__dict__"))
        mutable._0 = "after"
        self.assertEqual(mutable._0, "after")

    def test_shared_module_contains_multiple_declarations(self) -> None:
        model = Both(first=First(), second=Second())
        self.assertIsInstance(model.first, First)
        self.assertIsInstance(model.second, Second)
        self.assertEqual(
            typing.get_type_hints(Both), {"first": First, "second": Second}
        )

    def test_external_enum_shapes(self) -> None:
        self.assertEqual(Status.Active.value, "Active")
        self.assertEqual(Status.OnHold.value, "on-hold")
        self.assertEqual(
            typing.get_args(Event), (EventCreated, EventMoved, EventDeleted)
        )
        self.assertEqual(
            [field.name for field in dataclasses.fields(EventLogin)], ["user_id"]
        )
        self.assertEqual(
            [field.name for field in dataclasses.fields(EventLogout)], ["user_id"]
        )
        self.assertIsInstance(EventLogin(user_id=1), EventLogin)
        self.assertNotIsInstance(EventLogin(user_id=1), EventLogout)
        self.assertEqual(
            typing.get_args(AuditEvent), (AuditEventCreated, AuditEventMutable)
        )

    def test_rust_documentation_is_available_at_runtime(self) -> None:
        self.assertEqual(
            LegacyModel.__doc__,
            "Legacy user model.\n\nRetained for migration.\n\n"
            "Deprecated: Use User (since 0.1.0).",
        )
        self.assertEqual(DocumentedStatus.__doc__, "Documented status values.")
        self.assertEqual(
            DocumentedEventCreated.__doc__,
            "An item was created.",
        )

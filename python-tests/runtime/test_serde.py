from __future__ import annotations

import dataclasses
import typing
import unittest

from binding.AdjacentlyTagged import (
    AdjacentlyTagged,
    AdjacentlyTaggedCount,
    AdjacentlyTaggedDetails,
    AdjacentlyTaggedDetailsContent,
    AdjacentlyTaggedReady,
)
from binding.FlattenedRecord import FlattenedRecord
from binding.InternallyTagged import (
    InternallyTagged,
    InternallyTaggedReady,
    InternallyTaggedWithData,
)
from binding.SerdeRecord import SerdeRecord
from binding.SerdeStatus import SerdeStatus
from binding.Status import Status
from binding.UntaggedValue import (
    UntaggedValue,
    UntaggedValueDetails,
    UntaggedValueText,
)
from binding.User import User


class SerdeContracts(unittest.TestCase):
    def test_required_and_skipped_constructor_fields(self) -> None:
        with self.assertRaises(TypeError):
            SerdeRecord(userId=1, reviewStatus=SerdeStatus.AwaitingReview)
        with self.assertRaises(TypeError):
            FlattenedRecord(request_id=7, displayName="Ada")
        with self.assertRaises(TypeError):
            User(
                id=1,
                display_name="Ada",
                address=None,
                status=Status.Active,
                events=[],
                labels={},
                secret="not-generated",
            )

    def test_serde_rename_and_flatten(self) -> None:
        record = SerdeRecord(
            userId=1,
            reviewStatus=SerdeStatus.AwaitingReview,
            explicit_name="Ada",
        )
        self.assertEqual(record.userId, 1)
        self.assertEqual(SerdeStatus.AwaitingReview.value, "AWAITING_REVIEW")

        flattened = FlattenedRecord(request_id=7, displayName="Ada", enabled=True)
        self.assertEqual(
            [field.name for field in dataclasses.fields(flattened)],
            ["request_id", "displayName", "enabled"],
        )

    def test_tagged_enum_discriminators_and_payloads(self) -> None:
        ready = InternallyTaggedReady()
        data = InternallyTaggedWithData(itemCount=2)
        self.assertEqual(
            typing.get_args(InternallyTagged),
            (InternallyTaggedReady, InternallyTaggedWithData),
        )
        self.assertEqual(ready.kind, "ready")
        self.assertEqual(data.kind, "with_data")
        self.assertEqual(
            typing.get_args(typing.get_type_hints(InternallyTaggedWithData)["kind"]),
            ("with_data",),
        )
        with self.assertRaises(TypeError):
            InternallyTaggedWithData(kind="wrong", itemCount=2)

        adjacent_ready = AdjacentlyTaggedReady()
        count = AdjacentlyTaggedCount(payload=3)
        details = AdjacentlyTaggedDetails(
            payload=AdjacentlyTaggedDetailsContent(item_id=9)
        )
        self.assertEqual(
            typing.get_args(AdjacentlyTagged),
            (AdjacentlyTaggedReady, AdjacentlyTaggedCount, AdjacentlyTaggedDetails),
        )
        self.assertEqual(adjacent_ready.kind, "ready")
        self.assertEqual(
            [field.name for field in dataclasses.fields(adjacent_ready)], ["kind"]
        )
        self.assertEqual(count.kind, "count")
        self.assertEqual(details.kind, "details")
        self.assertEqual(details.payload.item_id, 9)
        details_hints = typing.get_type_hints(AdjacentlyTaggedDetails)
        self.assertIs(details_hints["payload"], AdjacentlyTaggedDetailsContent)

    def test_untagged_variants(self) -> None:
        text = UntaggedValueText("value")
        details = UntaggedValueDetails(value=3)
        self.assertEqual(text._0, "value")
        self.assertEqual(details.value, 3)
        self.assertEqual(
            typing.get_args(UntaggedValue),
            (UntaggedValueText, UntaggedValueDetails),
        )
        self.assertNotIn("kind", typing.get_type_hints(UntaggedValueDetails))

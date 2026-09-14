from __future__ import annotations

import collections
import datetime
import decimal
import ipaddress
import pathlib
import typing
import unittest
import uuid

from binding.FeatureCoverage import FeatureCoverage
from binding.JsonPayload import JsonPayload
from binding.Nested import Nested
from binding.ResultCoverage import ResultCoverage
from binding.TypeCoverage import TypeCoverage
from binding.ZeroArray import ZeroArray


class TypeMappingContracts(unittest.TestCase):
    def test_result_exposes_only_success_values(self) -> None:
        self.assertEqual(
            typing.get_type_hints(ResultCoverage),
            {
                "item": Nested,
                "items": list[Nested],
                "optional": str | None,
                "overridden": str,
            },
        )
        item = Nested(id=1)
        model = ResultCoverage(item=item, items=[item], optional=None, overridden="id")
        self.assertIs(model.item, item)

    def test_standard_type_annotations_resolve(self) -> None:
        self.assertEqual(
            typing.get_type_hints(TypeCoverage),
            {
                "queue": collections.deque[Nested],
                "linked": list[bool],
                "heap": list[int],
                "ordered_set": set[str],
                "hashed_set": set[int],
                "ordered_map": dict[str, Nested],
                "hashed_map": dict[str, Nested],
                "borrowed": Nested,
                "cow": str,
                "cow_slice": list[int],
                "cow_nested": list[Nested],
                "path": pathlib.Path,
                "ip": ipaddress.IPv4Address | ipaddress.IPv6Address,
                "socket": str,
                "amount": decimal.Decimal,
                "one": tuple[int],
                "three": tuple[int, str, Nested],
                "array": tuple[Nested, Nested],
                "long_array": tuple[int, ...],
            },
        )
        self.assertEqual(typing.get_type_hints(ZeroArray), {"empty": tuple[()]})

    def test_optional_feature_annotations_resolve(self) -> None:
        hints = typing.get_type_hints(FeatureCoverage)
        expected = {
            "duration": datetime.timedelta,
            "system_time": datetime.datetime,
            "immutable_bytes": bytes,
            "mutable_bytes": bytearray,
            "date": datetime.date,
            "time": datetime.time,
            "naive_datetime": datetime.datetime,
            "utc_datetime": datetime.datetime,
            "fixed_datetime": datetime.datetime,
            "timezone_datetime": datetime.datetime,
            "url": str,
            "uuid": uuid.UUID,
            "decimal": decimal.Decimal,
            "object_id": str,
            "bson_uuid": uuid.UUID,
            "index_map": dict[str, int],
            "index_set": list[str],
            "ordered": float,
            "not_nan": float,
            "bounded_vec": list[int],
            "bounded_deque": list[str],
            "bounded_string": str,
            "version": str,
            "version_requirement": str,
            "small_string": str,
            "async_mutex": int,
            "async_rw_lock": str,
            "async_once": int | None,
        }
        self.assertEqual({name: hints[name] for name in expected}, expected)
        self.assertIn("json", hints)

    def test_json_alias_is_complete_and_recursive(self) -> None:
        hints = typing.get_type_hints(JsonPayload)
        self.assertEqual(typing.get_origin(hints["map"]), dict)
        self.assertEqual(typing.get_args(hints["map"])[0], str)

        variants = typing.get_args(hints["value"])
        self.assertEqual(variants[:5], (type(None), bool, int, float, str))
        list_alias, dict_alias = variants[5:]
        self.assertIs(typing.get_origin(list_alias), list)
        self.assertIs(typing.get_origin(dict_alias), dict)
        recursive = typing.get_args(list_alias)[0]
        # Python 3.10 retains a string here; newer versions wrap it in ForwardRef.
        self.assertIn(
            recursive, ("_PyRsJsonValue", typing.ForwardRef("_PyRsJsonValue"))
        )
        self.assertEqual(typing.get_args(dict_alias), (str, recursive))

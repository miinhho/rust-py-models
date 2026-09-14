from __future__ import annotations

import pathlib
import typing
import unittest

from binding.Choice import Choice, ChoiceMany, ChoiceOne
from binding.Item import Item
from binding.Page import Page
from binding.Pair import Pair
from binding.Root import Root


class GenericContracts(unittest.TestCase):
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

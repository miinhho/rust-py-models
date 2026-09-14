from __future__ import annotations

import pathlib
import typing
import unittest

from binding.Choice import Choice, ChoiceMany, ChoiceOne
from binding.Item import Item
from binding.MapHolder import MapHolder
from binding.Outer import Outer
from binding.Page import Page
from binding.Pair import Pair
from binding.ProjectedLeft import ProjectedLeft
from binding.ProjectedRight import ProjectedRight
from binding.ProjectedRoot import ProjectedRoot
from binding.ResultPage import ResultPage
from binding.ResultRoot import ResultRoot
from binding.Root import Root


class GenericContracts(unittest.TestCase):
    def test_grouped_projection_keeps_only_python_parameters(self) -> None:
        hints = typing.get_type_hints(ProjectedRoot)
        self.assertEqual(hints["outer"], Outer[Item])
        self.assertEqual(hints["left"], ProjectedLeft[Item])
        self.assertEqual(hints["map"], MapHolder[int, str])
        self.assertEqual(len(Outer.__parameters__), 1)
        self.assertEqual(len(ProjectedLeft.__parameters__), 1)
        self.assertEqual(len(MapHolder.__parameters__), 2)
        self.assertEqual(
            typing.get_type_hints(ProjectedLeft)["right"],
            ProjectedRight[ProjectedLeft.__parameters__[0]] | None,
        )

    def test_result_error_parameter_is_erased(self) -> None:
        self.assertEqual(typing.get_type_hints(ResultRoot)["page"], ResultPage[Item])
        self.assertEqual(len(ResultPage.__parameters__), 1)
        self.assertEqual(
            typing.get_type_hints(ResultPage),
            {
                "value": ResultPage.__parameters__[0],
                "history": list[ResultPage.__parameters__[0]],
            },
        )

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

from __future__ import annotations

import typing
import unittest

from binding.CycleA import CycleA
from binding.CycleB import CycleB
from binding.CycleC import CycleC
from binding.graph.left.NestedCycleA import NestedCycleA
from binding.graph.NestedCycleC import NestedCycleC
from binding.graph.right.NestedCycleB import NestedCycleB
from binding.Left import Left
from binding.Right import Right


class GraphContracts(unittest.TestCase):
    def test_cyclic_annotations_resolve_from_every_node(self) -> None:
        self.assertEqual(typing.get_type_hints(Left)["right"], Right | None)
        self.assertEqual(typing.get_type_hints(Right)["left"], Left | None)
        self.assertEqual(typing.get_type_hints(CycleA)["b"], CycleB | None)
        self.assertEqual(typing.get_type_hints(CycleB)["c"], CycleC | None)
        self.assertEqual(typing.get_type_hints(CycleC)["a"], CycleA | None)
        self.assertEqual(
            typing.get_type_hints(NestedCycleA)["b"],
            NestedCycleB | None,
        )
        self.assertEqual(
            typing.get_type_hints(NestedCycleB)["c"],
            NestedCycleC | None,
        )
        self.assertEqual(
            typing.get_type_hints(NestedCycleC)["a"],
            NestedCycleA | None,
        )

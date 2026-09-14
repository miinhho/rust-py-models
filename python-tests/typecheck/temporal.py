"""Static contracts for generated temporal annotations."""

from __future__ import annotations

import datetime
from typing import assert_type

from binding.FeatureCoverage import FeatureCoverage


def valid_usage(model: FeatureCoverage) -> None:
    assert_type(model.duration, datetime.timedelta)
    assert_type(model.system_time, datetime.datetime)
    assert_type(model.time, datetime.time)
    assert_type(model.naive_datetime, datetime.datetime)
    assert_type(model.utc_datetime, datetime.datetime)
    assert_type(model.fixed_datetime, datetime.datetime)
    assert_type(model.timezone_datetime, datetime.datetime)

    aware = datetime.datetime.now(datetime.timezone.utc)
    naive = aware.replace(tzinfo=None)
    model.duration = datetime.timedelta(seconds=1)
    model.system_time = aware
    model.naive_datetime = naive
    model.utc_datetime = aware
    model.fixed_datetime = aware
    model.timezone_datetime = aware


def rejected_usage(model: FeatureCoverage) -> None:
    model.duration = datetime.datetime.now()  # type: ignore[assignment]
    model.time = datetime.timedelta(seconds=1)  # type: ignore[assignment]
    model.system_time = datetime.date.today()  # type: ignore[assignment]

"""Static consumer contracts for generated Serde-compatible shapes."""

from __future__ import annotations

from binding.AdjacentlyTagged import (
    AdjacentlyTagged,
    AdjacentlyTaggedCount,
    AdjacentlyTaggedReady,
)
from binding.FlattenedRecord import FlattenedRecord
from binding.InternallyTagged import (
    InternallyTagged,
    InternallyTaggedReady,
    InternallyTaggedWithData,
)
from binding.models.Address import Address
from binding.SerdeRecord import SerdeRecord
from binding.SerdeStatus import SerdeStatus
from binding.UntaggedValue import (
    UntaggedValue,
    UntaggedValueDetails,
    UntaggedValueText,
)


def valid_usage() -> None:
    internal: InternallyTagged = InternallyTaggedWithData(itemCount=1)
    adjacent: AdjacentlyTagged = AdjacentlyTaggedReady()
    adjacent_count: AdjacentlyTagged = AdjacentlyTaggedCount(payload=1)
    untagged_text: UntaggedValue = UntaggedValueText("text")
    untagged_details: UntaggedValue = UntaggedValueDetails(value=1)
    serde_record = SerdeRecord(
        userId=1,
        reviewStatus=SerdeStatus.AwaitingReview,
        explicit_name="Ada",
    )
    flattened = FlattenedRecord(request_id=1, displayName="Ada", enabled=True)
    _ = (
        internal,
        adjacent,
        adjacent_count,
        untagged_text,
        untagged_details,
        serde_record,
        flattened,
    )


def rejected_usage() -> None:
    address = Address(city="Seoul", zip=None)
    invalid_internal: InternallyTagged = address  # type: ignore[assignment]
    invalid_adjacent: AdjacentlyTagged = UntaggedValueText("value")  # type: ignore[assignment]
    _ = (invalid_internal, invalid_adjacent)
    InternallyTaggedWithData(kind="wrong", itemCount=1)  # type: ignore[call-arg]
    SerdeRecord(  # type: ignore[call-arg]
        user_id=1,
        reviewStatus=SerdeStatus.AwaitingReview,
        explicit_name="Ada",
    )
    SerdeRecord(  # type: ignore[call-arg]
        userId=1,
        reviewStatus=SerdeStatus.AwaitingReview,
    )
    FlattenedRecord(request_id=1, displayName="Ada")  # type: ignore[call-arg]
    _ = InternallyTaggedReady()

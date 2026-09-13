"""Static consumer contract for generated classes, checked by mypy only."""

from __future__ import annotations

from bindings.Event import Event, EventCreated, EventDeleted, EventMoved
from bindings.ImmutableUser import ImmutableUser
from bindings.Left import Left
from bindings.models.Address import Address
from bindings.Page import Page
from bindings.Right import Right
from bindings.Status import Status
from bindings.User import User


def valid_usage() -> None:
    address = Address(city="Seoul", zip=None)
    event: Event = EventMoved(_0=address)
    user = User(
        id=1,
        display_name="Ada",
        address=address,
        status=Status.Active,
        events=[event, EventDeleted()],
        labels={"role": "admin"},
    )
    optional_address: Address | None = user.address
    page: Page[int] = Page(value=1, items=[1, None])
    left = Left(right=Right(left=None))
    immutable = ImmutableUser(id=1, name="Ada")
    _ = (optional_address, page, left, immutable)


def event_user_id(event: Event) -> int | None:
    if isinstance(event, EventCreated):
        return event.user_id
    return None


def rejected_usage() -> None:
    # Each expected error must stay on its own line. An unused ignore fails mypy.
    Address(city=1, zip=None)  # type: ignore[arg-type]
    Address(city="Seoul")  # type: ignore[call-arg]
    EventCreated(user_id="wrong")  # type: ignore[arg-type]
    invalid_event: Event = Address(city="Seoul", zip=None)  # type: ignore[assignment]
    invalid_status: Status = "Active"  # type: ignore[assignment]
    _ = (invalid_event, invalid_status)
    Page[int](value="wrong", items=[])  # type: ignore[arg-type]
    ImmutableUser(1, "Ada")  # type: ignore[call-arg]
    immutable = ImmutableUser(id=1, name="Ada")
    immutable.name = "Grace"  # type: ignore[misc]
    User(
        id=1,
        display_name="Ada",
        address="wrong",  # type: ignore[arg-type]
        status=Status.Active,
        events=[],
        labels={},
    )
    User(
        id=1,
        display_name="Ada",
        status=Status.Active,
        events=[],
        labels={},
    )  # type: ignore[call-arg]

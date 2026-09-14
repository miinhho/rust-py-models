"""Static consumer contracts for generated model shapes."""

from __future__ import annotations

from binding.AuditEvent import AuditEvent, AuditEventCreated, AuditEventMutable
from binding.Event import Event, EventCreated, EventDeleted, EventMoved
from binding.ImmutableUser import ImmutableUser
from binding.Item import Item
from binding.Left import Left
from binding.models.Address import Address
from binding.Nested import Nested
from binding.Page import Page
from binding.ResultCoverage import ResultCoverage
from binding.ResultPage import ResultPage
from binding.ResultRoot import ResultRoot
from binding.Right import Right
from binding.Status import Status
from binding.User import User


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
    mutable_event: AuditEvent = AuditEventMutable("value")
    result = ResultCoverage(
        item=Nested(id=1), items=[Nested(id=2)], optional=None, overridden="id"
    )
    result_page: ResultPage[Item] = ResultPage(value=Item(id=1), history=[])
    result_root = ResultRoot(page=result_page)
    _ = (
        user,
        optional_address,
        page,
        left,
        immutable,
        mutable_event,
        result,
        result_root,
    )


def event_user_id(event: Event) -> int | None:
    if isinstance(event, EventCreated):
        return event.user_id
    return None


def rejected_usage() -> None:
    # Each expected error must stay on its own line. An unused ignore fails mypy.
    Address(city=1, zip=None)  # type: ignore[arg-type]
    ResultCoverage(item="wrong", items=[], optional=None, overridden="id")  # type: ignore[arg-type]
    ResultPage[Item](value="wrong", history=[])  # type: ignore[arg-type]
    Address(city="Seoul")  # type: ignore[call-arg]
    EventCreated(user_id="wrong")  # type: ignore[arg-type]
    invalid_event: Event = Address(city="Seoul", zip=None)  # type: ignore[assignment]
    invalid_status: Status = "Active"  # type: ignore[assignment]
    _ = (invalid_event, invalid_status)
    Page[int](value="wrong", items=[])  # type: ignore[arg-type]
    ImmutableUser(1, "Ada")  # type: ignore[call-arg]
    AuditEventCreated(1)  # type: ignore[call-arg]
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

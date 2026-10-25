"""CAVEAT Language for Python: a decision ledger for agents.

A Caveat program records what a decision rests on. Values carry their
evidence and caveats, a decision keeps the grounds it was made on, and when
evidence is withdrawn the decisions that rested on it reopen.

This package runs the same Rust runtime as the ``caveat-lang`` npm package,
compiled for Python instead of WebAssembly, and returns the same bytes::

    import caveat_lang as caveat

    with caveat.open(source) as session:
        outcome = session.dispatch("observe", {"confidence": 82})
        if isinstance(outcome, caveat.Rejected):
            print(outcome.code, outcome.message)
        else:
            print(outcome.snapshot["bindings"])

Outcomes follow Dispatch 0.1 (spec/caveat-dispatch-0.1.md): an accepted or
rejected event is a returned value, and anything else raises ``FatalError``,
after which the session cannot be used.
"""

from __future__ import annotations

import json
import math
import operator
import threading
from collections.abc import Mapping
from dataclasses import dataclass
from html import escape
from typing import Any, Callable, ClassVar, Dict, Iterable, List, Literal, Optional, Tuple, TypeVar, Union

from ._display import state_html
from ._runtime import NativeSession, RuntimeFailure, Trap

__all__ = [
    "DISPATCH_SCHEMA",
    "ORIGINS",
    "VIEW_FIELDS",
    "Accepted",
    "AcceptedDelta",
    "AcceptedView",
    "CaveatError",
    "ClosedError",
    "FatalError",
    "LoadError",
    "Origin",
    "PayloadError",
    "Rejected",
    "RestoreError",
    "Session",
    "SessionState",
    "ThreadError",
    "ViewField",
    "check",
    "decision_history",
    "interface",
    "open",
    "payload_text",
    "restore",
]

__version__ = "0.1.0"

DISPATCH_SCHEMA = "caveat-dispatch/0.1"

Origin = Literal["policy", "input", "evaluation", "limit"]
ORIGINS: Tuple[Origin, ...] = ("policy", "input", "evaluation", "limit")

ViewField = Literal[
    "bindings",
    "binding_explanations",
    "cues",
    "effects",
    "commitments",
    "commitment_grounds",
    "decision_series",
    "decision_journal",
    "relations",
]
# The view fields dispatch_view can select, in the view's order.
VIEW_FIELDS: Tuple[ViewField, ...] = (
    "bindings",
    "binding_explanations",
    "cues",
    "effects",
    "commitments",
    "commitment_grounds",
    "decision_series",
    "decision_journal",
    "relations",
)

SessionState = Literal["open", "closed", "fatal"]
ViewSchema = Literal["0.1", "0.2"]
JsonObject = Dict[str, Any]
Payload = Mapping[str, object]


class CaveatError(Exception):
    """Base class of every error this package raises on purpose.

    Catch the subclasses. ``kind`` names each as the JavaScript kit's
    ``CaveatError.kind`` does: "load", "restore", "payload", "fatal",
    "closed" or "thread".
    """

    kind: ClassVar[str] = "caveat"


class LoadError(CaveatError):
    """The program did not load."""

    kind = "load"


class RestoreError(CaveatError):
    """The save does not restore with this source."""

    kind = "restore"


class PayloadError(CaveatError, ValueError):
    """The payload or event name cannot be sent as JSON unchanged. The session is untouched."""

    kind = "payload"


class FatalError(CaveatError):
    """The session failed and cannot be used again.

    ``report`` is the runtime's fatal report (``code`` and ``message``) when
    the failure carried one.
    """

    kind = "fatal"

    def __init__(self, message: str, report: Optional[JsonObject] = None) -> None:
        super().__init__(message)
        self.report = report


class ClosedError(CaveatError):
    """The session was closed."""

    kind = "closed"


class ThreadError(CaveatError):
    """The session was used from a thread other than the one that made it. Nothing changed."""

    kind = "thread"


@dataclass(frozen=True, repr=False)
class Accepted:
    """An accepted event, from ``Session.dispatch``: the snapshot after it."""

    snapshot: JsonObject
    accepted: ClassVar[Literal[True]] = True
    # The snapshot is a dict, so an outcome compares by value but cannot be hashed.
    __hash__ = None  # type: ignore[assignment]

    def __repr__(self) -> str:
        return f"<Accepted {self.snapshot.get('last_event')!r}, event {self.snapshot.get('sequence')}>"

    def _repr_html_(self) -> str:
        return state_html(self.snapshot)


@dataclass(frozen=True, repr=False)
class AcceptedView:
    """An accepted event, from ``Session.dispatch_view``: the view after it."""

    view: JsonObject
    accepted: ClassVar[Literal[True]] = True
    __hash__ = None  # type: ignore[assignment]

    def __repr__(self) -> str:
        return f"<AcceptedView {self.view.get('last_event')!r}, event {self.view.get('sequence')}>"

    def _repr_html_(self) -> str:
        return state_html(self.view)


@dataclass(frozen=True, repr=False)
class AcceptedDelta:
    """An accepted event, from ``Session.dispatch_view_delta``: how View 0.2 changed."""

    delta: JsonObject
    accepted: ClassVar[Literal[True]] = True
    __hash__ = None  # type: ignore[assignment]

    def __repr__(self) -> str:
        return f"<AcceptedDelta, event {self.delta.get('sequence')}>"


@dataclass(frozen=True)
class Rejected:
    """A classified refusal. The session is unchanged.

    Only a "policy"/"reject" message is stable: it is the program's quoted
    literal.
    """

    origin: Origin
    code: str
    message: str
    accepted: ClassVar[Literal[False]] = False


# --- payloads ---------------------------------------------------------------

_SAFE_INTEGER = 2**53


def _number_text(value: float) -> str:
    """``value`` as JavaScript's ``JSON.stringify`` writes it.

    The runtime reads ``1`` and ``1.0`` as different numbers, so a payload
    must be written as the JavaScript kit writes it for the two to agree.
    Both languages find the same shortest digits; only the layout differs
    (ECMAScript Number::toString).
    """
    if value == 0:
        return "0"
    sign = "-" if value < 0 else ""
    mantissa, _, exponent = repr(abs(value)).partition("e")
    whole, _, fraction = mantissa.partition(".")
    digits = (whole + fraction).lstrip("0").rstrip("0")
    # ECMAScript's n: the value is 0.DIGITS times ten to the power `point`.
    point = int(exponent or 0)
    if whole == "0":
        point -= len(fraction) - len(fraction.lstrip("0"))
    else:
        point += len(whole)
    count = len(digits)
    if count <= point <= 21:
        return sign + digits + "0" * (point - count)
    if 0 < point <= 21:
        return sign + digits[:point] + "." + digits[point:]
    if -6 < point <= 0:
        return sign + "0." + "0" * -point + digits
    shown = digits[0] + ("." + digits[1:] if count > 1 else "")
    power = point - 1
    return f"{sign}{shown}e{'+' if power > 0 else '-'}{abs(power)}"


def _string_text(value: str, at: str) -> str:
    try:
        value.encode("utf-8")
    except UnicodeEncodeError:
        raise PayloadError(f"payload {at or 'value'} has a lone surrogate, which UTF-8 cannot carry") from None
    return json.dumps(value, ensure_ascii=False)


def _write(value: object, at: str, out: List[str]) -> None:
    if value is None:
        out.append("null")
    elif value is True:
        out.append("true")
    elif value is False:
        out.append("false")
    elif isinstance(value, str):
        out.append(_string_text(value, at))
    elif isinstance(value, float):
        if not math.isfinite(value):
            raise PayloadError(f"payload {at or 'value'} is not a finite number")
        out.append(_number_text(value))
    elif isinstance(value, (list, tuple)):
        out.append("[")
        for index, item in enumerate(value):
            if index:
                out.append(",")
            _write(item, f"{at}/{index}", out)
        out.append("]")
    elif isinstance(value, Mapping):
        out.append("{")
        for index, (key, item) in enumerate(value.items()):
            if not isinstance(key, str):
                raise PayloadError(f"payload {at or 'value'} has a key that is not a string: {key!r}")
            if index:
                out.append(",")
            out.append(_string_text(key, at))
            out.append(":")
            _write(item, f"{at}/{key}", out)
        out.append("}")
    else:
        _write_integer(value, at, out)


def _write_integer(value: object, at: str, out: List[str]) -> None:
    """An int, or anything that is one (such as numpy.int64), as JSON."""
    try:
        integer = operator.index(value)  # type: ignore[arg-type]
    except TypeError:
        raise PayloadError(f"payload {at or 'value'} cannot be sent as JSON: {type(value).__name__}") from None
    if abs(integer) > _SAFE_INTEGER:
        raise PayloadError(f"payload {at or 'value'} is an integer JSON numbers cannot hold exactly")
    out.append(_number_text(float(integer)))


def payload_text(payload: object) -> str:
    """The payload as JSON text, with the meaning the JavaScript kit's would have.

    Accepts None, bools, strings, ints (and integer types such as
    numpy.int64), floats, lists, tuples and mappings with string keys.
    Numbers are written as JavaScript writes them. Raises ``PayloadError``
    for anything else, for a float that is not finite, for an int beyond
    2**53, which JSON numbers do not hold exactly, and for a string UTF-8
    cannot carry. Negative zero is written as 0, as JavaScript writes it.
    """
    out: List[str] = []
    _write(payload, "", out)
    return "".join(out)


# --- reading the runtime's text ---------------------------------------------


def _fatal_from(error: Exception) -> FatalError:
    text = str(error)
    if isinstance(error, Trap):
        return FatalError(f"the runtime panicked: {text}")
    try:
        report = json.loads(text)
    except ValueError:
        return FatalError(text)
    if isinstance(report, dict) and report.get("schema") == DISPATCH_SCHEMA and report.get("outcome") == "fatal":
        return FatalError(f"{report.get('code')}: {report.get('message')}", report)
    return FatalError(text)


def _parsed(text: str, what: str) -> Any:
    """The runtime's JSON text, parsed. Text that is not JSON is fatal."""
    try:
        return json.loads(text)
    except ValueError:
        raise FatalError(f"{what} returned text that is not JSON") from None


Outcome = TypeVar("Outcome", Accepted, AcceptedView, AcceptedDelta)


def _outcome(text: str, field: str, accepted: Callable[[JsonObject], Outcome]) -> Union[Outcome, Rejected]:
    """Read one dispatch outcome. Anything Dispatch 0.1 does not allow is fatal."""
    outcome = _parsed(text, "dispatch")
    if isinstance(outcome, dict) and outcome.get("schema") == DISPATCH_SCHEMA:
        if outcome.get("outcome") == "accepted" and isinstance(outcome.get(field), dict):
            return accepted(outcome[field])
        if outcome.get("outcome") == "rejected":
            origin, code, message = outcome.get("origin"), outcome.get("code"), outcome.get("message")
            if origin in ORIGINS and isinstance(code, str) and isinstance(message, str):
                return Rejected(origin, code, message)
    raise FatalError(f"unrecognised dispatch outcome: {text[:200]}")


def _fields_text(fields: Iterable[ViewField]) -> str:
    chosen: List[str] = []
    for field in fields:
        if field not in VIEW_FIELDS:
            raise ValueError(f"unknown view field {field!r}; the fields are {', '.join(VIEW_FIELDS)}")
        if field in chosen:
            raise ValueError(f"view field {field!r} is named twice")
        chosen.append(field)
    return json.dumps(chosen)


def _event_text(event: str) -> str:
    if not isinstance(event, str) or not event:
        raise ValueError("event must be a non-empty string")
    _string_text(event, "event name")
    return event


# --- sessions ---------------------------------------------------------------

Value = TypeVar("Value")


class Session:
    """One running program. Make one with ``open`` or ``restore``.

    A session is open until ``close`` (or the end of a ``with`` block) closes
    it, or a fatal outcome makes it unusable. A session belongs to the thread
    that made it; a call from any other thread raises ``ThreadError``.
    """

    def __init__(self, native: NativeSession) -> None:
        self._native: Optional[NativeSession] = native
        self._state: SessionState = "open"
        self._thread = threading.get_ident()

    def __enter__(self) -> Session:
        return self

    def __exit__(self, *exc_info: object) -> None:
        self.close()

    def __repr__(self) -> str:
        return f"<caveat_lang.Session {self._state}>"

    def _repr_html_(self) -> str:
        if self._state == "open" and self._thread == threading.get_ident():
            try:
                return state_html(self.view())
            except CaveatError:
                pass
        return f"<code>{escape(repr(self))}</code>"

    @property
    def state(self) -> SessionState:
        """"open", "closed" or "fatal"."""
        return self._state

    def _usable(self) -> NativeSession:
        if self._thread != threading.get_ident():
            raise ThreadError("this session belongs to another thread; open a session in each thread")
        if self._state == "closed":
            raise ClosedError("session is closed")
        if self._state == "fatal" or self._native is None:
            raise FatalError("session is unusable after a fatal outcome")
        return self._native

    def _fail(self, error: Exception) -> FatalError:
        self._state = "fatal"
        self._native = None
        return error if isinstance(error, FatalError) else _fatal_from(error)

    def _call(self, read: Callable[[NativeSession], Value]) -> Value:
        """Call the runtime. Any failure there is fatal to the session."""
        native = self._usable()
        try:
            return read(native)
        except (RuntimeFailure, Trap) as error:
            raise self._fail(error) from None

    def _json(self, read: Callable[[NativeSession], str], what: str) -> Any:
        text = self._call(read)
        try:
            return _parsed(text, what)
        except FatalError as error:
            raise self._fail(error) from None

    def _dispatch(
        self,
        send: Callable[[NativeSession, str, str], str],
        field: str,
        accepted: Callable[[JsonObject], Outcome],
        event: str,
        payload: Optional[Payload],
    ) -> Union[Outcome, Rejected]:
        self._usable()
        event = _event_text(event)
        text = payload_text({} if payload is None else payload)
        raw = self._call(lambda native: send(native, event, text))
        try:
            return _outcome(raw, field, accepted)
        except FatalError as error:
            raise self._fail(error) from None

    def dispatch(self, event: str, payload: Optional[Payload] = None) -> Union[Accepted, Rejected]:
        """Send one event and return ``Accepted`` with the snapshot, or ``Rejected``.

        The payload holds exactly the parameters the program declares for the
        event; an event without parameters takes none. Anything else is
        rejected as input/payload_invalid. A payload JSON cannot carry raises
        ``PayloadError`` and changes nothing. Any other failure raises
        ``FatalError``.
        """
        return self._dispatch(
            lambda native, name, text: native.dispatch_outcome(name, text), "snapshot", Accepted, event, payload
        )

    def dispatch_view(
        self, event: str, payload: Optional[Payload] = None, *, fields: Optional[Iterable[ViewField]] = None
    ) -> Union[AcceptedView, Rejected]:
        """``dispatch``, with the View 0.1 after the event in place of the snapshot.

        ``fields`` selects view fields (see ``VIEW_FIELDS``); the view then
        holds only those, plus schema, sequence and last_event. An unknown or
        repeated name raises ``ValueError`` before the event is sent.
        """
        if fields is None:
            return self._dispatch(
                lambda native, name, text: native.dispatch_view_outcome(name, text),
                "view", AcceptedView, event, payload,
            )
        chosen = _fields_text(fields)

        def send(native: NativeSession, name: str, text: str) -> str:
            return native.dispatch_view_fields_outcome(name, text, chosen)

        return self._dispatch(send, "view", AcceptedView, event, payload)

    def dispatch_view_delta(self, event: str, payload: Optional[Payload] = None) -> Union[AcceptedDelta, Rejected]:
        """``dispatch``, with the View 0.2 delta of the event in place of the snapshot.

        Applied to ``view(schema="0.2")`` from before the event, the delta
        gives the view after it.
        """
        return self._dispatch(
            lambda native, name, text: native.dispatch_view_delta_outcome(name, text), "delta", AcceptedDelta, event,
            payload,
        )

    def snapshot(self) -> JsonObject:
        """The session's full state, as a new dict."""
        snapshot: JsonObject = self._json(lambda native: native.snapshot_compact(), "snapshot")
        return snapshot

    def snapshot_text(self) -> str:
        """The snapshot as the runtime's pretty JSON text."""
        return self._call(lambda native: native.snapshot())

    def view(self, schema: ViewSchema = "0.1") -> JsonObject:
        """The per-event view: View 0.1, or View 0.2 with ``schema="0.2"``."""
        view: JsonObject = self._json(self._view_reader(schema), "view")
        return view

    def view_text(self, schema: ViewSchema = "0.1") -> str:
        """The view as the runtime's JSON text."""
        return self._call(self._view_reader(schema))

    @staticmethod
    def _view_reader(schema: ViewSchema) -> Callable[[NativeSession], str]:
        if schema == "0.1":
            return lambda native: native.view()
        if schema == "0.2":
            return lambda native: native.view_v2()
        raise ValueError("view schema must be '0.1' or '0.2'")

    def save(self, *, compact: bool = False, positional: bool = False) -> str:
        """The session as JSON text to store. Restore it with the exact source it came from.

        ``compact=True`` gives the replay save while the accepted events fit
        in one and it is smaller, otherwise the state save. With
        ``positional=True`` as well it is the positional replay save, which
        holds more events.
        """
        if positional and not compact:
            raise ValueError("positional applies to a compact save; pass compact=True as well")
        if positional:
            text = self._call(lambda native: native.save_compact_positional())
        elif compact:
            text = self._call(lambda native: native.save_compact())
        else:
            text = self._call(lambda native: native.save())
        try:
            _parsed(text, "save")
        except FatalError as error:
            raise self._fail(error) from None
        return text

    def drain_archive(self) -> List[JsonObject]:
        """The archive items departed since the last drain (spec/caveat-departure-0.1.md).

        The archive is not saved: a host that keeps exact history stores
        these beside its saves.
        """
        items = self._json(lambda native: native.drain_archive(), "drain_archive")
        if not isinstance(items, list):
            raise self._fail(FatalError("drain_archive returned JSON that is not a list"))
        return items

    @property
    def undrained(self) -> int:
        """How many archive items wait to be drained."""
        return self._call(lambda native: native.undrained())

    def close(self) -> None:
        """Close the session. Closing twice, or after a fatal outcome, does nothing."""
        self._native = None
        if self._state == "open":
            self._state = "closed"


def _source_text(source: str) -> str:
    if not isinstance(source, str):
        raise TypeError("source must be text (str)")
    return source


def _program_call(read: Callable[[], str], failure: Callable[[str], CaveatError]) -> str:
    """Call the runtime about a program, outside any session."""
    try:
        return read()
    except RuntimeFailure as error:
        raise failure(str(error)) from None
    except Trap as error:
        raise _fatal_from(error) from None


def open(source: str) -> Session:  # noqa: A001 - read as caveat.open, as gzip.open is
    """Load a program and start a session of it. Raises ``LoadError`` if it does not load."""
    text = _source_text(source)
    try:
        return Session(NativeSession(text))
    except RuntimeFailure as error:
        raise LoadError(str(error)) from None
    except Trap as error:
        raise _fatal_from(error) from None


def restore(source: str, save: str) -> Session:
    """Resume a session from ``Session.save`` text, with the exact source it came from.

    Raises ``RestoreError`` when the save does not restore with this source.
    """
    text = _source_text(source)
    if not isinstance(save, str):
        raise TypeError("save must be text (str)")
    try:
        return Session(NativeSession.restore(text, save))
    except RuntimeFailure as error:
        raise RestoreError(str(error)) from None
    except Trap as error:
        raise _fatal_from(error) from None


def check(source: str, *, unused: bool = False, records: bool = False, save: Optional[str] = None) -> JsonObject:
    """Advisory warnings for a program that loads (spec/caveat-check-0.1.md).

    ``unused=True`` adds C007 and ``records=True`` C008. ``save`` (its text),
    with ``records=True``, adds C009 for that save. A program that does not
    load raises ``LoadError``; a save that does not restore, ``RestoreError``.
    """
    text = _source_text(source)
    if save is not None and not isinstance(save, str):
        raise TypeError("save must be text (str)")
    if save is not None and not records:
        raise ValueError("a save is checked with records=True")
    if save is not None:
        # The program must load first, so its error is reported as a load error.
        open(text).close()
        restore(text, save).close()
    if records:
        options: Dict[str, object] = {"unused": unused, "records": records}
        if save is not None:
            options["save"] = save
        report = _program_call(lambda: NativeSession.check_options(text, json.dumps(options)), LoadError)
    elif unused:
        report = _program_call(lambda: NativeSession.check_with(text, True), LoadError)
    else:
        report = _program_call(lambda: NativeSession.check(text), LoadError)
    parsed: JsonObject = _parsed(report, "check")
    return parsed


def interface(source: str) -> JsonObject:
    """The program's events, states, bindings and declared names (spec/caveat-interface-0.1.md)."""
    text = _source_text(source)
    parsed: JsonObject = _parsed(_program_call(lambda: NativeSession.interface(text), LoadError), "interface")
    return parsed


def _journal_change(entry: JsonObject) -> JsonObject:
    change = {
        "sequence": entry["sequence"],
        "event": entry["event"],
        "because": list(entry["because"]),
        "caveats": list(entry["caveats"]),
    }
    for key in ("value", "permitted_by"):
        if key in entry:
            change[key] = entry[key]
    return change


def decision_history(view: JsonObject, decision: str) -> List[JsonObject]:
    """Every revision of one decision series, oldest first, read from a view or snapshot.

    Each revision says what it was committed on and, once reopened, what
    reopened it. Its status is "in_force", "reopened" or "superseded". A
    revision whose journal entries a window has retired is listed with
    ``retired: True`` and None for each change the journal no longer holds.
    The same as the JavaScript kit's ``decisionHistory``.
    """
    series = (view.get("decision_series") or {}).get(decision)
    if not series:
        raise KeyError(f"{decision} is not a decision series of this program")
    revisions: Dict[str, JsonObject] = {
        item["id"]: {"revision": item["id"], "status": "superseded", "retired": True, "committed": None,
                     "reopened": None}
        for item in series["revisions"]
    }
    for entry in view["decision_journal"]:
        revision = revisions.get(entry["commitment"]) if entry.get("decision") == decision else None
        if revision is None:
            continue
        if entry["change"] == "committed":
            revision["committed"] = _journal_change(entry)
            revision["retired"] = False
        elif entry["change"] == "reopened":
            # A hold's "held", "deferred" and "released" entries reopen nothing.
            revision["reopened"] = _journal_change(entry)
    current = revisions.get(series["current"]) if series.get("current") is not None else None
    if current is not None:
        commitment = next((item for item in view["commitments"] if item.get("action") == series["current"]), None)
        # View 0.1 and the snapshot say "open"; View 0.2 says "reopened".
        if commitment is None:
            reopened = True
        else:
            reopened = commitment.get("open")
            if reopened is None:
                reopened = commitment.get("reopened")
        current["status"] = "in_force" if reopened is False else "reopened"
    return list(revisions.values())

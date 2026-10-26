"""How a snapshot or view shows itself in a notebook: its bindings, and each
decision with what it rests on and what reopened it."""

from __future__ import annotations

from html import escape
from typing import Any, Dict, Iterable, List

_STYLE = "border-collapse:collapse;margin:0.25em 0"
# A table shows this many rows, then says how many more there are.
_ROWS = 50
_CELL = "border:1px solid #8884;padding:2px 8px;text-align:left;vertical-align:top"


def _table(caption: str, header: Iterable[str], rows: List[List[str]]) -> str:
    head = "".join(f'<th style="{_CELL}">{escape(name)}</th>' for name in header)
    shown = rows[:_ROWS]
    body = "".join("<tr>" + "".join(f'<td style="{_CELL}">{cell}</td>' for cell in row) + "</tr>" for row in shown)
    if len(rows) > len(shown):
        body += f'<tr><td style="{_CELL}" colspan="{len(shown[0])}">&hellip; {len(rows) - len(shown)} more</td></tr>'
    return f'<table style="{_STYLE}"><caption style="text-align:left"><b>{escape(caption)}</b></caption>' \
           f"<tr>{head}</tr>{body}</table>"


def _names(names: Iterable[Any]) -> str:
    shown = [f"<code>{escape(str(name))}</code>" for name in names]
    return ", ".join(shown) if shown else "&mdash;"


def _flatten(prefix: str, value: Any, out: List[List[str]]) -> None:
    if isinstance(value, dict) and value:
        for key, item in value.items():
            _flatten(f"{prefix}.{key}" if prefix else str(key), item, out)
    else:
        out.append([f"<code>{escape(prefix)}</code>", escape(repr(value))])


def state_html(state: Dict[str, Any]) -> str:
    """HTML for a snapshot or a view (View 0.1)."""
    parts = [f"<div><b>Caveat</b> after event {escape(str(state.get('sequence')))} "
             f"(<code>{escape(str(state.get('last_event')))}</code>)</div>"]
    bindings: List[List[str]] = []
    _flatten("", state.get("bindings") or {}, bindings)
    if bindings:
        parts.append(_table("Bindings", ["name", "value"], bindings))
    # A view names a decision's grounds commitment_grounds; a snapshot keeps
    # them, with the value, in commitment_bases.
    grounds = state.get("commitment_grounds")
    if grounds is None:
        bases = state.get("commitment_bases") or {}
        grounds = {action: base.get("provenance", {}) for action, base in bases.items()}
    decisions = []
    for commitment in state.get("commitments") or []:
        action = commitment.get("action")
        reopened = commitment.get("open")
        if reopened is None:
            reopened = commitment.get("reopened")
        basis = grounds.get(action, {})
        decisions.append([
            f"<code>{escape(str(action))}</code>",
            "reopened" if reopened else "in force",
            _names(basis.get("evidence", [])),
            _names(basis.get("caveats", [])),
            _names(commitment.get("reopened_by", [])),
        ])
    if decisions:
        parts.append(_table("Decisions", ["decision", "status", "rests on", "caveats", "reopened by"], decisions))
    return "".join(parts)

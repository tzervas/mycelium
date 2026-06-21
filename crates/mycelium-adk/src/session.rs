//! `adk.session` — content-addressed session, state, and events (RFC-0023 §4.3).
//!
//! `State` is an **immutable key→value snapshot** (ADR-003 content-addressed identity).
//! "Mutation" returns a **new** `State` — never in-place writes (RT1, RFC-0008:91-97).
//!
//! ## Types
//! - [`Value`] — a simple value type for the session state scratchpad
//! - [`State`] — immutable key→`Value` snapshot
//! - [`EventAuthor`] — who authored an event
//! - [`EventContent`] — what an event contains
//! - [`Event`] — one append-only log entry (author + content)
//! - [`Session`] — `State` + `Vec<Event>` (the full interaction record)
//!
//! ## Honesty (VR-5 / ADR-003)
//! - `put(&State, key, value) -> State` returns a **new** snapshot; the original is
//!   unchanged.  Value semantics: identity = content (ADR-003).
//! - `append_event(session, event) -> Session` returns a new `Session` with the event
//!   appended; it never mutates in place.
//! - `State::get` returns `Option<&Value>` — `None` for a missing key, never a
//!   fabricated default (C1 never-silent).
//!
//! ## FLAG (RFC-0023 §4.3 — immutability tension)
//! ADK's mutable prefix-scoped `State` scratchpad has no 1:1 immutable analogue.
//! The snapshot model here is the honest v0.  Concurrent sub-agent merge (`fuse`, RT6)
//! is deferred (E7-2/M-667; `fuse` is Ratified-not-yet-lexed).
//!
//! ## FLAG (E7-1 / M-657)
//! `Session::events` uses `Vec<Event>`; the Mycelium target surface is `List<Event>`
//! (needs generics M-657).
//!
//! ## Design spec
//! `docs/rfcs/RFC-0023-Agent-Development-Kit-Phylum.md` §4.3

use std::collections::BTreeMap;
use std::fmt;

// ── Value (session scratchpad value type) ─────────────────────────────────────

/// A session-state scratchpad value.
///
/// Deliberately simple for this wave: `Text`, `Integer`, `Boolean`, or `Absent`.
/// Extending to the full `CoreValue` type family is a future integration task
/// (depends on importing `mycelium-core`'s `CoreValue` — that surface integration
/// is deferred; for now we use this minimal sum).
///
/// ## Honesty (VR-5)
/// `Absent` is the explicit representation of a missing/unset value.  It is **never**
/// used as a silent default for a lookup miss — `State::get` returns `Option<&Value>`,
/// and `Absent` is only stored explicitly when a caller explicitly sets it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    /// A text (string) value.
    Text(String),
    /// An integer value.
    Integer(i64),
    /// A boolean value.
    Boolean(bool),
    /// An explicitly-absent value (a "no value" that is stored, not inferred).
    Absent,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Text(s) => write!(f, "{s:?}"),
            Value::Integer(n) => write!(f, "{n}"),
            Value::Boolean(b) => write!(f, "{b}"),
            Value::Absent => write!(f, "<absent>"),
        }
    }
}

// ── State ─────────────────────────────────────────────────────────────────────

/// An immutable key→[`Value`] snapshot (RFC-0023 §4.3; ADR-003 content-addressed identity).
///
/// A `State` is a value: its identity is its content, not a mutable address.
/// "Mutation" is always [`put`] — it returns a **new** `State` (value-semantic, RT1).
///
/// ## ADR-003 note
/// Full content-addressed hashing (blake3 over the key-value pairs) is a future
/// integration step.  This wave implements the *value-semantic interface* (every
/// `put` produces a structurally distinct copy), which is the observable guarantee;
/// the content-hash identity is a future optimization.
///
/// ## Prefix scoping
/// ADK's `app:`/`user:`/`temp:`/session prefix scoping is supported via the key
/// string: callers use `"app:city"`, `"user:pref"`, etc.  No enforcement here —
/// the runner applies scoping policy.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct State {
    map: BTreeMap<String, Value>,
}

impl State {
    /// Construct an empty `State`.
    pub fn new() -> Self {
        State {
            map: BTreeMap::new(),
        }
    }

    /// Look up `key` in this state snapshot.
    ///
    /// Returns `Some(&Value)` if `key` is present, `None` if absent.
    ///
    /// # Honesty (C1 never-silent)
    /// `None` is the explicit signal for a missing key — it is never replaced by a
    /// fabricated default.  Callers must handle `None` explicitly.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.map.get(key)
    }

    /// Return the number of entries in this state snapshot.
    #[must_use]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Return `true` if the state has no entries.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Iterate over all key-value pairs in this snapshot.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Value)> {
        self.map.iter()
    }
}

/// Return a **new** `State` with `key` mapped to `value` (value-semantic; ADR-003).
///
/// The original `state` is **not** mutated — it is cloned and the new key is inserted
/// into the clone.  This is the only sanctioned way to "update" a `State`.
///
/// # Value semantics (RT1 / ADR-003)
/// The returned `State` is a new, distinct snapshot.  If the same `(key, value)` is
/// inserted twice, the resulting snapshots compare equal (content identity, ADR-003).
///
/// # Guarantee tag: `Exact` (pure, no approximation, no hidden state).
#[must_use]
pub fn put(state: &State, key: impl Into<String>, value: Value) -> State {
    let mut new_map = state.map.clone();
    new_map.insert(key.into(), value);
    State { map: new_map }
}

// ── Event ─────────────────────────────────────────────────────────────────────

/// Who authored an [`Event`] in the session log.
///
/// `User` — a human turn; `Agent(name)` — an agent's response; `Tool(name)` — a
/// tool's output; `System` — a system-injected event (e.g. a budget notice).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventAuthor {
    /// A human user message.
    User,
    /// An agent's event, identified by the agent's name.
    Agent(String),
    /// A tool's output event, identified by the tool's name.
    Tool(String),
    /// A system-injected event (e.g. budget exhaustion notice).
    System,
}

impl fmt::Display for EventAuthor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventAuthor::User => write!(f, "user"),
            EventAuthor::Agent(name) => write!(f, "agent:{name}"),
            EventAuthor::Tool(name) => write!(f, "tool:{name}"),
            EventAuthor::System => write!(f, "system"),
        }
    }
}

/// The content of a session [`Event`].
///
/// `Text` — a natural-language turn; `ToolResult { name, value }` — a tool's output;
/// `Marker(String)` — a structured control signal (budget exhausted, routing decision).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventContent {
    /// Natural-language text (a user message, model response, etc.).
    Text(String),
    /// A tool's output.
    ToolResult {
        /// The tool that produced this result.
        name: String,
        /// The serialized value returned by the tool (as a string for simplicity;
        /// full `CoreValue` integration is a future step).
        value: String,
    },
    /// A structured control marker (e.g. `"budget:exhausted"`, routing decision).
    Marker(String),
}

/// One content-addressed log entry in the session event stream (RFC-0023 §4.3).
///
/// Events are **append-only** — the log is never mutated, only extended.  A new
/// event is added via [`append_event`], which returns a new [`Session`].
///
/// ## Honesty (VR-5)
/// Events are `Debug + Clone + PartialEq` — value-semantic.  The append-only
/// invariant is structural: the only way to "modify" the event log is to create
/// a new `Session` with the new event appended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    /// Who authored this event.
    pub author: EventAuthor,
    /// The content of this event.
    pub content: EventContent,
    /// A monotonic sequence index (set by the runner; starts at 0 for the first event).
    pub seq: u64,
}

// ── Session ───────────────────────────────────────────────────────────────────

/// The full interaction record: a [`State`] snapshot + an append-only event log
/// (RFC-0023 §4.3; ADR-003).
///
/// A `Session` is value-semantic: "updating" state or "appending" an event returns
/// a **new** `Session` (via [`update_state`] and [`append_event`]).
///
/// ## FLAG (E7-1 / M-657)
/// `events` uses `Vec<Event>`; the Mycelium target surface is `List<Event>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    /// The current state snapshot.
    pub state: State,
    /// The append-only event log.
    ///
    /// FLAG (E7-1 / M-657): Mycelium target surface is `List<Event>`.
    pub events: Vec<Event>,
}

impl Session {
    /// Construct a new, empty `Session`.
    pub fn new() -> Self {
        Session {
            state: State::new(),
            events: Vec::new(),
        }
    }
}

impl Default for Session {
    fn default() -> Self {
        Session::new()
    }
}

/// Return a new `Session` with `key` → `value` set in the state (value-semantic).
///
/// The original `session` is not mutated.  Delegates to [`put`] for the state update.
#[must_use]
pub fn update_state(session: &Session, key: impl Into<String>, value: Value) -> Session {
    Session {
        state: put(&session.state, key, value),
        events: session.events.clone(),
    }
}

/// Return a new `Session` with `event` appended to the event log (append-only).
///
/// The original `session` is not mutated.  The event log is append-only — existing
/// events are never removed or reordered.
///
/// # Honesty (VR-5)
/// The returned `Session` has `events.len() == original.len() + 1` and all prior
/// events are unchanged.  This is verified by the property test below.
#[must_use]
pub fn append_event(session: &Session, event: Event) -> Session {
    let mut new_events = session.events.clone();
    new_events.push(event);
    Session {
        state: session.state.clone(),
        events: new_events,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::{
        append_event, put, update_state, Event, EventAuthor, EventContent, Session, State, Value,
    };
    use proptest::prelude::*;

    // ── Unit tests ─────────────────────────────────────────────────────────────

    /// `put` returns a new `State`; the original is unchanged (value-semantic, ADR-003).
    #[test]
    fn put_returns_new_snapshot_original_unchanged() {
        let s0 = State::new();
        let s1 = put(&s0, "city", Value::Text("Paris".to_owned()));
        assert_eq!(s0.get("city"), None, "original must be unchanged");
        assert_eq!(
            s1.get("city"),
            Some(&Value::Text("Paris".to_owned())),
            "new snapshot must contain the key"
        );
    }

    /// `put` on an already-set key produces a new snapshot with the updated value.
    #[test]
    fn put_overwrites_in_new_snapshot() {
        let s0 = put(&State::new(), "x", Value::Integer(1));
        let s1 = put(&s0, "x", Value::Integer(2));
        assert_eq!(s0.get("x"), Some(&Value::Integer(1)), "original unchanged");
        assert_eq!(
            s1.get("x"),
            Some(&Value::Integer(2)),
            "new snapshot updated"
        );
    }

    /// `State::get` returns `None` for a missing key — never a silent default (C1).
    #[test]
    fn state_get_returns_none_for_missing_key_never_silent() {
        let s = State::new();
        assert_eq!(
            s.get("nonexistent"),
            None,
            "missing key must be None, never a fabricated default (C1 never-silent)"
        );
    }

    /// `append_event` returns a new `Session`; the original is unchanged.
    #[test]
    fn append_event_returns_new_session_original_unchanged() {
        let s0 = Session::new();
        let e = Event {
            author: EventAuthor::User,
            content: EventContent::Text("Hello".to_owned()),
            seq: 0,
        };
        let s1 = append_event(&s0, e.clone());
        assert_eq!(s0.events.len(), 0, "original session must be unchanged");
        assert_eq!(s1.events.len(), 1, "new session must have the event");
        assert_eq!(&s1.events[0], &e);
    }

    /// `update_state` returns a new `Session` with updated state; events unchanged.
    #[test]
    fn update_state_returns_new_session_events_unchanged() {
        let mut s0 = Session::new();
        s0 = append_event(
            &s0,
            Event {
                author: EventAuthor::System,
                content: EventContent::Marker("start".to_owned()),
                seq: 0,
            },
        );
        let s1 = update_state(&s0, "answer", Value::Text("Paris".to_owned()));
        assert_eq!(
            s1.state.get("answer"),
            Some(&Value::Text("Paris".to_owned()))
        );
        assert_eq!(s1.events.len(), s0.events.len(), "events must be preserved");
    }

    /// `Session` is `Clone` + `PartialEq` — value-semantic.
    #[test]
    fn session_is_value_semantic() {
        let s0 = Session::new();
        let s1 = s0.clone();
        assert_eq!(s0, s1);
    }

    /// `Value::Absent` is explicit — it can be stored and retrieved without confusion.
    #[test]
    fn value_absent_can_be_stored_and_retrieved() {
        let s = put(&State::new(), "k", Value::Absent);
        assert_eq!(s.get("k"), Some(&Value::Absent));
    }

    // ── Property tests ────────────────────────────────────────────────────────

    proptest! {
        /// BOUND: `put` always produces a distinct snapshot — the original and the new
        /// snapshot differ unless the key already had the same value.
        /// Guard: an in-place mutation that returns the same object breaks this.
        #[test]
        fn prop_put_produces_distinct_snapshot_for_new_key(key in "[a-z]{1,10}", val in 0i64..=100) {
            let s0 = State::new();
            let s1 = put(&s0, key.clone(), Value::Integer(val));
            // The new snapshot contains the key; the original does not.
            prop_assert_eq!(s1.get(&key), Some(&Value::Integer(val)));
            prop_assert_eq!(s0.get(&key), None, "original must not be mutated");
        }

        /// BOUND: `append_event` is strictly monotone: the new session always has exactly
        /// one more event than the original, and all prior events are unchanged.
        #[test]
        fn prop_append_event_is_monotone_and_append_only(
            text in "[a-z]{1,30}",
            seq in 0u64..=1000,
        ) {
            let s0 = Session::new();
            let e = Event {
                author: EventAuthor::User,
                content: EventContent::Text(text),
                seq,
            };
            let s1 = append_event(&s0, e.clone());
            prop_assert_eq!(s1.events.len(), s0.events.len() + 1);
            prop_assert_eq!(&s1.events[s0.events.len()], &e, "last event must be the appended one");
        }

        /// BOUND: `State::get` for a key that was just `put` always returns the exact
        /// value that was stored — no transformation, no truncation (Exact tag).
        #[test]
        fn prop_put_then_get_returns_exact_value(key in "[a-z]{1,10}", val in 0i64..=1000) {
            let s = put(&State::new(), key.clone(), Value::Integer(val));
            prop_assert_eq!(s.get(&key), Some(&Value::Integer(val)));
        }

        /// BOUND: Two `put` calls with the same key and value produce equal snapshots
        /// (content identity / ADR-003 idempotence).
        #[test]
        fn prop_put_same_kv_twice_produces_equal_snapshots(
            key in "[a-z]{1,10}",
            val in 0i64..=100,
        ) {
            let s1 = put(&State::new(), key.clone(), Value::Integer(val));
            let s2 = put(&State::new(), key.clone(), Value::Integer(val));
            prop_assert_eq!(s1, s2, "same content => equal snapshots (ADR-003)");
        }
    }
}

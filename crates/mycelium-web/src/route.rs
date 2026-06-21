//! nodule: `web.route` — a reified, inspectable route table + dispatch with mandatory EXPLAIN
//! (C3), per RFC-0022 §4.1 / §4.5.
//!
//! TODO(leaf WEB / M-670): a reified `RouteTable`; `match_route(&table, &method, &path) ->
//! Result<(Handler, PathParams), RouteError>` with `RouteError { NotFound, MethodNotAllowed
//! { allowed } }` — an explicit 404/405, **never a silent wrong-handler**; and an inspectable
//! `RouteMatch` record (which pattern + captures) so every dispatch is EXPLAIN-able. Pure;
//! `Exact`-when-`Ok`.

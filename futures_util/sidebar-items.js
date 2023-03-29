window.SIDEBAR_ITEMS = {"macro":[["join","Polls multiple futures simultaneously, returning a tuple of all results once complete."],["pending","A macro which yields to the event loop once."],["pin_mut","Pins a value on the stack."],["poll","A macro which returns the result of polling a future once within the current `async` context."],["ready","Extracts the successful type of a `Poll<T>`."],["select","Polls multiple futures and streams simultaneously, executing the branch for the future that finishes first. If multiple futures are ready, one will be pseudo-randomly selected at runtime. Futures directly passed to `select!` must be `Unpin` and implement `FusedFuture`."],["select_biased","Polls multiple futures and streams simultaneously, executing the branch for the future that finishes first. Unlike `select!`, if multiple futures are ready, one will be selected in order of declaration. Futures directly passed to `select_biased!` must be `Unpin` and implement `FusedFuture`."],["stream_select","Combines several streams, all producing the same `Item` type, into one stream. This is similar to `select_all` but does not require the streams to all be the same type. It also keeps the streams inline, and does not require `Box<dyn Stream>`s to be allocated. Streams passed to this macro must be `Unpin`."],["try_join","Polls multiple futures simultaneously, resolving to a [`Result`] containing either a tuple of the successful outputs or an error."]],"mod":[["future","Asynchronous values."],["lock","Futures-powered synchronization primitives."],["never","This module contains the `Never` type."],["sink","Asynchronous sinks."],["stream","Asynchronous streams."],["task","Tools for working with tasks."]]};
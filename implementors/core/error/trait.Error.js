(function() {var implementors = {
"base64":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"base64/enum.DecodeError.html\" title=\"enum base64::DecodeError\">DecodeError</a>"]],
"bitvec":[["impl&lt;R&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"bitvec/index/struct.BitIdxError.html\" title=\"struct bitvec::index::BitIdxError\">BitIdxError</a>&lt;R&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;R: <a class=\"trait\" href=\"bitvec/mem/trait.BitRegister.html\" title=\"trait bitvec::mem::BitRegister\">BitRegister</a>,</span>"],["impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"bitvec/ptr/struct.MisalignError.html\" title=\"struct bitvec::ptr::MisalignError\">MisalignError</a>&lt;T&gt;"],["impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"bitvec/ptr/enum.BitPtrError.html\" title=\"enum bitvec::ptr::BitPtrError\">BitPtrError</a>&lt;T&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"bitvec/store/trait.BitStore.html\" title=\"trait bitvec::store::BitStore\">BitStore</a>,</span>"],["impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"bitvec/ptr/enum.BitSpanError.html\" title=\"enum bitvec::ptr::BitSpanError\">BitSpanError</a>&lt;T&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"bitvec/store/trait.BitStore.html\" title=\"trait bitvec::store::BitStore\">BitStore</a>,</span>"]],
"clap_builder":[["impl&lt;F:&nbsp;<a class=\"trait\" href=\"clap_builder/error/trait.ErrorFormatter.html\" title=\"trait clap_builder::error::ErrorFormatter\">ErrorFormatter</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"clap_builder/error/struct.Error.html\" title=\"struct clap_builder::error::Error\">Error</a>&lt;F&gt;"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"clap_builder/parser/enum.MatchesError.html\" title=\"enum clap_builder::parser::MatchesError\">MatchesError</a>"]],
"crossbeam_channel":[["impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"crossbeam_channel/struct.SendError.html\" title=\"struct crossbeam_channel::SendError\">SendError</a>&lt;T&gt;"],["impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"crossbeam_channel/enum.TrySendError.html\" title=\"enum crossbeam_channel::TrySendError\">TrySendError</a>&lt;T&gt;"],["impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"crossbeam_channel/enum.SendTimeoutError.html\" title=\"enum crossbeam_channel::SendTimeoutError\">SendTimeoutError</a>&lt;T&gt;"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"crossbeam_channel/struct.RecvError.html\" title=\"struct crossbeam_channel::RecvError\">RecvError</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"crossbeam_channel/enum.TryRecvError.html\" title=\"enum crossbeam_channel::TryRecvError\">TryRecvError</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"crossbeam_channel/enum.RecvTimeoutError.html\" title=\"enum crossbeam_channel::RecvTimeoutError\">RecvTimeoutError</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"crossbeam_channel/struct.TrySelectError.html\" title=\"struct crossbeam_channel::TrySelectError\">TrySelectError</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"crossbeam_channel/struct.SelectTimeoutError.html\" title=\"struct crossbeam_channel::SelectTimeoutError\">SelectTimeoutError</a>"]],
"crypto_common":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"crypto_common/struct.InvalidLength.html\" title=\"struct crypto_common::InvalidLength\">InvalidLength</a>"]],
"digest":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"digest/struct.InvalidOutputSize.html\" title=\"struct digest::InvalidOutputSize\">InvalidOutputSize</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"digest/struct.InvalidBufferSize.html\" title=\"struct digest::InvalidBufferSize\">InvalidBufferSize</a>"]],
"either":[["impl&lt;L, R&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"either/enum.Either.html\" title=\"enum either::Either\">Either</a>&lt;L, R&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;L: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;R: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a>,</span>"]],
"futures_task":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"futures_task/struct.SpawnError.html\" title=\"struct futures_task::SpawnError\">SpawnError</a>"]],
"futures_util":[["impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/any/trait.Any.html\" title=\"trait core::any::Any\">Any</a>, Item&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"futures_util/stream/struct.ReuniteError.html\" title=\"struct futures_util::stream::ReuniteError\">ReuniteError</a>&lt;T, Item&gt;"],["impl&lt;T, E:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/fmt/trait.Display.html\" title=\"trait core::fmt::Display\">Display</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"futures_util/stream/struct.TryChunksError.html\" title=\"struct futures_util::stream::TryChunksError\">TryChunksError</a>&lt;T, E&gt;"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"futures_util/future/struct.Aborted.html\" title=\"struct futures_util::future::Aborted\">Aborted</a>"]],
"getrandom":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"getrandom/struct.Error.html\" title=\"struct getrandom::Error\">Error</a>"]],
"http":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"http/header/struct.InvalidHeaderName.html\" title=\"struct http::header::InvalidHeaderName\">InvalidHeaderName</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"http/header/struct.InvalidHeaderValue.html\" title=\"struct http::header::InvalidHeaderValue\">InvalidHeaderValue</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"http/header/struct.ToStrError.html\" title=\"struct http::header::ToStrError\">ToStrError</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"http/method/struct.InvalidMethod.html\" title=\"struct http::method::InvalidMethod\">InvalidMethod</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"http/status/struct.InvalidStatusCode.html\" title=\"struct http::status::InvalidStatusCode\">InvalidStatusCode</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"http/uri/struct.InvalidUri.html\" title=\"struct http::uri::InvalidUri\">InvalidUri</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"http/uri/struct.InvalidUriParts.html\" title=\"struct http::uri::InvalidUriParts\">InvalidUriParts</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"http/struct.Error.html\" title=\"struct http::Error\">Error</a>"]],
"httparse":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"httparse/enum.Error.html\" title=\"enum httparse::Error\">Error</a>"]],
"idna":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"idna/struct.Errors.html\" title=\"struct idna::Errors\">Errors</a>"]],
"itertools":[["impl&lt;I&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"itertools/structs/struct.ExactlyOneError.html\" title=\"struct itertools::structs::ExactlyOneError\">ExactlyOneError</a>&lt;I&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;I: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/iter/traits/iterator/trait.Iterator.html\" title=\"trait core::iter::traits::iterator::Iterator\">Iterator</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;I::<a class=\"associatedtype\" href=\"https://doc.rust-lang.org/1.68.2/core/iter/traits/iterator/trait.Iterator.html#associatedtype.Item\" title=\"type core::iter::traits::iterator::Iterator::Item\">Item</a>: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a>,</span>"]],
"proc_macro2":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"proc_macro2/struct.LexError.html\" title=\"struct proc_macro2::LexError\">LexError</a>"]],
"rand":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"rand/distributions/enum.BernoulliError.html\" title=\"enum rand::distributions::BernoulliError\">BernoulliError</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"rand/distributions/weighted/enum.WeightedError.html\" title=\"enum rand::distributions::weighted::WeightedError\">WeightedError</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"rand/rngs/adapter/struct.ReadError.html\" title=\"struct rand::rngs::adapter::ReadError\">ReadError</a>"]],
"rand_core":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"rand_core/struct.Error.html\" title=\"struct rand_core::Error\">Error</a>"]],
"rayon_core":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"rayon_core/struct.ThreadPoolBuildError.html\" title=\"struct rayon_core::ThreadPoolBuildError\">ThreadPoolBuildError</a>"]],
"rustix":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"rustix/io/struct.Errno.html\" title=\"struct rustix::io::Errno\">Errno</a>"]],
"serde":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"serde/de/value/struct.Error.html\" title=\"struct serde::de::value::Error\">Error</a>"]],
"serde_json":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"serde_json/struct.Error.html\" title=\"struct serde_json::Error\">Error</a>"]],
"strsim":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"strsim/enum.StrSimError.html\" title=\"enum strsim::StrSimError\">StrSimError</a>"]],
"tokio":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"tokio/net/tcp/struct.ReuniteError.html\" title=\"struct tokio::net::tcp::ReuniteError\">ReuniteError</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"tokio/net/unix/struct.ReuniteError.html\" title=\"struct tokio::net::unix::ReuniteError\">ReuniteError</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"tokio/task/struct.JoinError.html\" title=\"struct tokio::task::JoinError\">JoinError</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"tokio/runtime/struct.TryCurrentError.html\" title=\"struct tokio::runtime::TryCurrentError\">TryCurrentError</a>"],["impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"tokio/sync/broadcast/error/struct.SendError.html\" title=\"struct tokio::sync::broadcast::error::SendError\">SendError</a>&lt;T&gt;"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"tokio/sync/broadcast/error/enum.RecvError.html\" title=\"enum tokio::sync::broadcast::error::RecvError\">RecvError</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"tokio/sync/broadcast/error/enum.TryRecvError.html\" title=\"enum tokio::sync::broadcast::error::TryRecvError\">TryRecvError</a>"],["impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"tokio/sync/mpsc/error/struct.SendError.html\" title=\"struct tokio::sync::mpsc::error::SendError\">SendError</a>&lt;T&gt;"],["impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"tokio/sync/mpsc/error/enum.TrySendError.html\" title=\"enum tokio::sync::mpsc::error::TrySendError\">TrySendError</a>&lt;T&gt;"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"tokio/sync/mpsc/error/enum.TryRecvError.html\" title=\"enum tokio::sync::mpsc::error::TryRecvError\">TryRecvError</a>"],["impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"tokio/sync/mpsc/error/enum.SendTimeoutError.html\" title=\"enum tokio::sync::mpsc::error::SendTimeoutError\">SendTimeoutError</a>&lt;T&gt;"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"tokio/sync/struct.TryLockError.html\" title=\"struct tokio::sync::TryLockError\">TryLockError</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"tokio/sync/oneshot/error/struct.RecvError.html\" title=\"struct tokio::sync::oneshot::error::RecvError\">RecvError</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"tokio/sync/oneshot/error/enum.TryRecvError.html\" title=\"enum tokio::sync::oneshot::error::TryRecvError\">TryRecvError</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"tokio/sync/struct.AcquireError.html\" title=\"struct tokio::sync::AcquireError\">AcquireError</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"tokio/sync/enum.TryAcquireError.html\" title=\"enum tokio::sync::TryAcquireError\">TryAcquireError</a>"],["impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"tokio/sync/enum.SetError.html\" title=\"enum tokio::sync::SetError\">SetError</a>&lt;T&gt;"],["impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"tokio/sync/watch/error/struct.SendError.html\" title=\"struct tokio::sync::watch::error::SendError\">SendError</a>&lt;T&gt;"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"tokio/sync/watch/error/struct.RecvError.html\" title=\"struct tokio::sync::watch::error::RecvError\">RecvError</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"tokio/time/error/struct.Error.html\" title=\"struct tokio::time::error::Error\">Error</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"tokio/time/error/struct.Elapsed.html\" title=\"struct tokio::time::error::Elapsed\">Elapsed</a>"]],
"tokio_stream":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"tokio_stream/struct.Elapsed.html\" title=\"struct tokio_stream::Elapsed\">Elapsed</a>"]],
"tokio_util":[["impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"tokio_util/sync/struct.PollSendError.html\" title=\"struct tokio_util::sync::PollSendError\">PollSendError</a>&lt;T&gt;"]],
"tungstenite":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"tungstenite/error/enum.Error.html\" title=\"enum tungstenite::error::Error\">Error</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"tungstenite/error/enum.CapacityError.html\" title=\"enum tungstenite::error::CapacityError\">CapacityError</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"tungstenite/error/enum.ProtocolError.html\" title=\"enum tungstenite::error::ProtocolError\">ProtocolError</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"tungstenite/error/enum.UrlError.html\" title=\"enum tungstenite::error::UrlError\">UrlError</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"tungstenite/error/enum.TlsError.html\" title=\"enum tungstenite::error::TlsError\">TlsError</a>"],["impl&lt;Role:&nbsp;<a class=\"trait\" href=\"tungstenite/handshake/trait.HandshakeRole.html\" title=\"trait tungstenite::handshake::HandshakeRole\">HandshakeRole</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"tungstenite/handshake/enum.HandshakeError.html\" title=\"enum tungstenite::handshake::HandshakeError\">HandshakeError</a>&lt;Role&gt;"]],
"url":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"url/enum.ParseError.html\" title=\"enum url::ParseError\">ParseError</a>"]],
"utf8":[["impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"utf8/enum.BufReadDecoderError.html\" title=\"enum utf8::BufReadDecoderError\">BufReadDecoderError</a>&lt;'a&gt;"],["impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.68.2/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"utf8/enum.DecodeError.html\" title=\"enum utf8::DecodeError\">DecodeError</a>&lt;'a&gt;"]]
};if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()
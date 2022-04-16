# warp-lifetime-issue-repro

Thank you for taking the time to investigate this issue.  There is either a
problem with my understanding of how this code should work, or with rustc's
interpretation of the code.  I have been unable to reproduce this issue outside
of warp or rocket-rs.

## Build environment

To ensure a consistent build environment, I use Docker images with podman.  The
following Docker images have all reproduced this issue:

* docker.io/library/rust:1.59-bullseye (d6c4db7b2530)
* docker.io/library/rust:1.60-bullseye (5593c6ce4c4e)
* docker.io/rustlang/rust:nightly-bullseye (cf477c958fa3 -- 1.62.0-nightly
(e7575f967 2022-04-14))

## Compiler output

```
error[E0308]: mismatched types
   --> src/main.rs:39:6
    |
39  |     .and_then(|_id, provider: Arc<T>| async move {
    |      ^^^^^^^^ lifetime mismatch
    |
    = note: expected type `for<'r> FnOnce<(&&Item,)>`
               found type `for<'r> FnOnce<(&'r &Item,)>`
note: this closure does not fulfill the lifetime requirements
   --> src/main.rs:42:41
    |
42  |         let items = items.iter().filter(|item| !item.is_deleted());
    |                                         ^^^^^^^^^^^^^^^^^^^^^^^^^
note: the lifetime requirement is introduced here
   --> /appsrc/.cache/registry/src/github.com-1ecc6299db9ec823/warp-0.3.2/src/filter/mod.rs:259:32
    |
259 |         F::Output: TryFuture + Send,
    |                                ^^^^
```

## Notes

The compiler specifically refers to the closure passed to `Iterator::filter()`
and calls its type `for<'r> FnOnce<(&'r &Item,)>`.  This makes no sense to me
as this isn't an `FnOnce` and `Iterator::filter()` doesn't even accept an
`FnOnce`, so I'm unsure where the compiler is getting this type from.

The compiler also specifically points to `F::Output` being bounded by `Send` as
the cause of the lifetime issue, but this makes little sense either, as this
closure is `Send`.

It's also worth noting that omitting the call to `p.can_see()` in
`write_items()` allows the code to compile, suggesting that the problem is
related to the computed lifetime of the future `write_items()` returns.

## Workarounds

### Free function instead of closure

Given this free function:

```
fn item_is_not_deleted(item: &&Item) -> bool {
    !item.is_deleted()
}
```

We can pass this to `Iterator::filter()`:

```
let items = items.iter().filter(item_is_not_deleted);
```

This satisfies the compiler, which hints that the lifetimes the compiler
deduces for the closure are incorrect and it should have deduced different
lifetimes.

### Inlining `write_items()`

If we manually inline `write_items()` into `demo()` the lifetime issue
disappears, hinting that the signature of this function could be responsible
for the problem.

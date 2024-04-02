# Payment engine

Assumptions:
1. Transaction cannot be disputed more than once.
2. Transactions are not processed for the locked account.

## Completeness

All cases are covered.

## Correctness

There are unit tests for major cases (mostly happy flow). Negative cases are
checked in the code and in the real project will be covered by tests.

## Safety and Robustness

No unsafe code, no unwraps and expects in the production code. Expects are only in the tests.
Using **Result** for the error handling together with **anyhow** crate.

## Efficiency

`Engine::apply_transactions` method accepts `std::io::Read` trait, so any type that implements
it can be used. That includes `std::fs::File` and `TcpStream`, so data can be streamed instead
of reading all at once.

To support multiple concurrent tcp streams `ClientData` should be wrapped in `Arc<Mutex<>>`.
Most likely there will be only 1 client per stream, so there should be no locks.

## Maintainability

In this case clean code is more important than efficient code because
humans will have to read and review your code without an opportunity for
you to explain it. Inefficient code can often be improved if it is correct and
highly maintainable

Code split in several modules each with its own logic.

Unit tests (including negative once that should be implemented in the actual project) will
guarantee correctness of the application logic during refactoring or extending.

## Possible improvements

1. `ClientData` could have a `state` with following variants: Normal, InDispute, Locked.
   This will allow to use type system for correct account processing (FSM approach).
2. Use `thiserror` crate for more precise error management (instead of current
   anyhow/string version).
3. Disputed (resolved or changed back) transactions can be removed from the
   `ClientData::transactions` map for more efficient memory usage.
4. `zeroize` crate can be used for safe data cleanup.

# 🚂 Choo Choo

[![Crates.io](https://img.shields.io/crates/v/choochoo.svg)](https://crates.io/crates/choochoo)
[![docs.rs](https://img.shields.io/docsrs/choochoo)](https://docs.rs/choochoo)
[![CI](https://github.com/azriel91/choochoo/workflows/CI/badge.svg)](https://github.com/azriel91/choochoo/actions/workflows/ci.yml)
[![Coverage Status](https://codecov.io/gh/azriel91/choochoo/branch/main/graph/badge.svg)](https://codecov.io/gh/azriel91/choochoo)

`choochoo` is a library that supports building operations tools with good user experience.

**Note:** This is still in early development, so expect frequent API breakages.

See:

* [`MOTIVATION.md`](MOTIVATION.md) for the motivation to create this library.
* [Operations UX](https://azriel.im/ops_ux/) for a book about the dimensions considered during `choochoo`'s design and development.
* The [examples](examples) directory for usage examples.


## Demo

https://user-images.githubusercontent.com/2993230/116825827-04b89c00-abe5-11eb-9e83-2a223f859ddd.mp4


## Features

| Symbol | Meaning          |
| :----: | ---------------- |
|   🟢   | Supported        |
|   🟡   | Work in progress |
|   ⚫   | Planned          |

* 🟢 Workflow graph with task dependencies
* 🟢 Concurrent task execution
* 🟢 Understandable error reporting (via [`codespan`](https://github.com/brendanzab/codespan))
* 🟢 Skip unnecessary work
* 🟢 Understandable progress
* 🟢 Actionable error messages
* 🟢 Namespaced working directory ([#21](https://github.com/azriel91/choochoo/issues/21))
* 🟡 Resource clean up ([#28](https://github.com/azriel91/choochoo/issues/28))
* 🟡 API Ergonomics and ease of doing the right thing.
* ⚫ Dry run
* ⚫ `choochoo` binary for configuration based workflows
* ⚫ Off-the-shelf support for common tasks
* ⚫ Web based UI
* ⚫ Agent mode to run `choochoo` on servers (Web API invocation)

Ideas which may be considered:

* Back up current state
* Restore previous state
* Telemetry logging for monitoring
* Metrics collection for analysis


## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.


### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

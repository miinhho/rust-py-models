# rust-py-models documentation

`rust-py-models` generates concrete Python model classes from Rust definitions. Start with the [quick start](../README.md), then use these guides for the details:

- [Generation and export](generation.md): derive, Python class shapes, attributes, dependencies, and output files.
- [Type mapping](type-mapping.md): built-in and feature-gated Rust types, plus semantic limits.
- [Compatibility](compatibility.md): supported Python versions and the checks run against generated files.

The generated classes are Python objects that callers can instantiate. Type annotations describe their intended fields; standard-library dataclasses do not enforce those annotations or implement Rust/Serde serialization.

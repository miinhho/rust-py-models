# Documentation

Start with the [quick start](../README.md) to generate and import a model. Use the reference that matches the question you are answering:

- [Generation and export](generation.md) — choose an export API, control Python names and paths, and understand generated class shapes.
- [Type mapping](type-mapping.md) — find the Python annotation emitted for a Rust type and its runtime limitations.

Generated classes are ordinary Python dataclasses and enums. Their annotations help type checkers; they do not perform Rust conversion, Serde serialization, or runtime validation.

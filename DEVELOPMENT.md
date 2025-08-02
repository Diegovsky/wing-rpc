# Regenerating tests
`cargo run -p wingc`: this parses and analyzes `.wing` files in the `test-files/` folder and regenerates the language definitions.

This is used by `cargo test -p wingc` to ensure a change does not break generated code. This test suite is not yet exhaustive!

# Running tests
`wingc/` has the most tests yet. It contains the parser, semantic analyzer and emitters.

Parser tests are run as **unit tests**.

Semantic emitter tests are run as **integration tests**.

There is unfortunately no tests for the semantic analyzer yet.

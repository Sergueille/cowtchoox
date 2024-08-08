
cargo r --bin doc-generator -- default/default.cowx docs/operators.cow
cargo r -- docs/operators.cow
mv docs/out.pdf docs/operators.pdf


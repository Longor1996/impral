default: testall

doc:
    cargo doc --no-deps --open

testall:
    cargo test --package impral --lib -- parser::tests::parse_all --exact --nocapture --ignored

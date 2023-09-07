default: testall

docs:
    cargo doc --no-deps -F guide

docs-open:
    cargo doc --no-deps -F guide --open

deps-update:
    cargo update

deps-check:
    cargo tree -e no-proc-macro

test:
    cargo test --package impral --lib -- --nocapture

testlex:
    cargo test --package impral --lib -- lexer2 --nocapture

testall:
    cargo test --package impral --lib -- parser::tests::parse_all --exact --nocapture --ignored

testhtml:
    cargo test --package impral --lib -- parser::tests::parse_into_html --exact --nocapture --ignored

build:
    cargo build

test:
    cargo test

doc:
    cargo doc "-Zunstable-options" "-Zrustdoc-scrape-examples" -p wing-rpc

[no-cd]
wingc *args:
    cargo run -p wingc-cli -- {{args}}

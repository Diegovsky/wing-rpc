build:
    cargo build

test:
    cargo test

doc:
    cargo doc "-Zunstable-options" "-Zrustdoc-scrape-examples" -p wing-rpc

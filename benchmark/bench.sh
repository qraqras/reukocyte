cargo clean && cargo build --release -p reukocyte
hyperfine --warmup 3 -i --runs 10 "target/release/reuko --only Layout/IndentationConsistency large_test.rb -f quiet" "rubocop --only Layout/IndentationConsistency large_test.rb -f quiet"
hyperfine --warmup 3 -i --runs 10 "target/release/reuko --only Layout/TrailingWhitespace large_test.rb -f quiet" "rubocop --only Layout/TrailingWhitespace large_test.rb -f quiet"

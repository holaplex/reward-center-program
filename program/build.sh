cargo build-bpf --bpf-out-dir ../target/deploy/
cp ../mpl-deps/mpl_auction_house-keypair.json ../target/deploy/mpl_auction_house-keypair.json
cp ../mpl-deps/mpl_token_metadata-keypair.json ../target/deploy/mpl_token_metadata-keypair.json
cp ../mpl-deps/mpl_auction_house.so ../target/deploy/mpl_auction_house.so
cp ../mpl-deps/mpl_token_metadata.so ../target/deploy/mpl_token_metadata.so
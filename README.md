proxify

A daemon that abstracts the usage of proxies (list) behind an API.

The daemon prepares a user given list of proxies, by connecting to a number of
them and making a HEAD request to make sure they are responsive. The purpose is
to eliminate timeouts when rotating proxies. The typical users of this daemon
are scrapers.

Data should be sent to the daemon on a socket using a binary structure:

(This is a preliminary structire, probably will change alot)

struct ProxifyData {
    session: u32,
    command: ProxifyCommand
    data: Vec<u8>,
}

This is also an attempt by me to become more proficient at writing Rust code,
so bare with me.

Example commands to test the build:
Server:
cargo run -- --debug=4 --config="bind_addr=127.0.0.1;bind_port=65432;proxies_file=proxies.json;nr_prepare_threads=5"
Client:
cargo run --example test_client
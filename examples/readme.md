# Examples

This examples shows how to use functionalities in this library.

## Decred daemon RPC client Example

This example shows how to use the rpcclient package to connect to a dcrd RPC
server using TLS-secured websockets and register for block connected and block
disconnected notifications till users signal a signal interrupt *subject to change*.

### Running the example

[Install](https://www.rust-lang.org/learn/get-started) the rust compiler. Next, modify the rpcclient.rs source to specify the correct username and password for the testnet rpc server.

```Rust
    user: "yourrpcuser".to_string(),
    password: "yourrpcpass".to_string(),
```

Finally, navigate to the parent directory where the `cargo.toml` file is located and run

```bash
$ RUST_LOG=trace cargo build --example rpcclient
```

## License

This example is licensed under the [copyfree](http://copyfree.org) ISC License.
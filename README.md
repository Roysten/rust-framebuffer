# rust-framebuffer
Basic framebuffer abstraction for Rust.

To be able to use this library, you have to add yourself to the `video` group:
```console
$ sudo usermod -aG video "$USER"
```

An example can be found in the examples directory. Use the following command to compile and run:
```console
$ cargo run --release --example rust-logo
```

Make sure to check out the starfield example as well!

Basic documentation is available: http://roysten.github.io/rust-framebuffer/target/doc/framebuffer/.
The documentation is a bit sparse, but I hope the examples can make up for that.

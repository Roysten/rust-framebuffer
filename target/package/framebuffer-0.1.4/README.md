# rust-framebuffer
Basic framebuffer abstraction for Rust.

An example can be found in the examples directory. Use the following command to compile and run:
```
sudo cargo run --release --example rust-logo
```

To avoid having to run all commands as root, you can add yourself to the video group:
```
sudo usermod -aG video <username>
```

Make sure to check out the starfield example as well!

Basic documentation is available: http://roysten.github.io/rust-framebuffer/target/doc/framebuffer/.
The documentation is a bit sparse, but I hope the examples can make up for that.

# GeneApp

To build for desktop
Run the following commands in order.

### install cargo bundle

```bash 
cargo install cargo-bundle
```

### build with cargo bundle

 This releases the application in 
 release mode with metadata specified in `cargo.toml` file.

```bash 
cargo bundle --release
```

Aftet build, check in directory `bundle` in `./target`

# mk (^∇^)-b


***

`mk` is a tool to run unix commands as another user, and a family of crates. It is similar to [`doas`](https://github.com/Duncaen/OpenDoas) or [`sudo`](https://github.com/sudo-project/sudo).

### Building

Rust's' `nightly` toolchain is required. The only version that this has been tested with is `1.56.0-nightly`.

The `build.sh` script will build `mk` and create a copy in the project's root, with permissions setup for testing.

***


### Disclaimers

 - I am not an expert, and this project is still very early in development. Don't use this anywhere except for testing.
 - Linux support is targeted first.

### License

`mk` and all its crates are distributed under the terms of the MIT license. See [LICENSE](LICENSE) for more information.

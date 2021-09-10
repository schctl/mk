# mk (^∇^)-b


`mk` is a tool to run commands as another user, and a family of crates. It is similar to [`doas`](https://github.com/Duncaen/OpenDoas) or [`sudo`](https://github.com/sudo-project/sudo).

---

## Building `mk`

### Requirements

 - Rust 1.56+
 - [Bindgen requirements](https://rust-lang.github.io/rust-bindgen/requirements.html)

### Feature flags

| Flag | Description |
|------|-------------|
| `pam` | Builds with authenticator support for [`PAM`](https://en.wikipedia.org/wiki/Pluggable_authentication_module) |
| `shadow` | Builds with authenticator support for reading shadow files |

`s.py` attempts to output a list of supported features on your system.

#### Building with supported features

```sh
cargo build --features $(./s.py)
```

---


### Disclaimers

 - I am not an expert, and this project is still very early in development. Don't use this anywhere except for testing.
 - Linux support is targeted first.

### License

`mk` and all its crates are distributed under the terms of the MIT license. See [LICENSE](LICENSE) for more information.

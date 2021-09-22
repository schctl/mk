# mk (^∇^)-b


`mk` is a tool to run commands as another user, and a family of crates. It is similar to [`doas`](https://github.com/Duncaen/OpenDoas) or [`sudo`](https://github.com/sudo-project/sudo).

---

## Building `mk`

### Requirements

 - Rust 1.56+
 - A C compiler
 - [Bindgen requirements](https://rust-lang.github.io/rust-bindgen/requirements.html)

### Feature flags

| Flag | Description |
|------|-------------|
| `pam` | Builds with for authentication using [`PAM`](https://en.wikipedia.org/wiki/Pluggable_authentication_module) (requires `libpam`) |
| `shadow` | Builds with support for authentication using the shadow password database |

## Configuration

`mk` searches for rules defined in `/etc/mk.conf`, configured in the [`TOML`](https://toml.io/en/) format.

---

### Disclaimer

I am not an expert, and this project is still very early in development. Don't use this anywhere except for testing.

### License

`mk` and all its crates are distributed under the terms of the MIT license. See [LICENSE](LICENSE) for more information.

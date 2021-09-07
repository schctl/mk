# mk (^∇^)-b


`mk` is a tool to run unix commands as another user, and a family of crates. It is similar to [`doas`](https://github.com/Duncaen/OpenDoas) or [`sudo`](https://github.com/sudo-project/sudo).

---

### Feature flags

| Flag | Description | Requires | Rust |
|------|-------------|----------|------|
| `pam` | Builds with authenticator support for [`PAM`](https://en.wikipedia.org/wiki/Pluggable_authentication_module) | A `PAM` implementation ([`Linux-PAM`](http://www.linux-pam.org/), [`OpenPAM`](https://www.openpam.org/)) | 1.56.0-nightly+  |
| `shadow` | Builds with authenticator support for [`shadow-utils`](https://github.com/shadow-maint/shadow) | System provided `shadow.h` | 1.45+ |

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

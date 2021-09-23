# mk (^∇^)-b

`mk` is a tool to run commands as another user. It is similar to [`doas`](https://github.com/Duncaen/OpenDoas) or [`sudo`](https://github.com/sudo-project/sudo).

---

## Building `mk`

### Requirements

- Rust 1.56+
- A C compiler
- [Bindgen requirements](https://rust-lang.github.io/rust-bindgen/requirements.html)

### Feature flags

| Flag     | Description                                                                                                                     |
| -------- | ------------------------------------------------------------------------------------------------------------------------------- |
| `pam`    | Builds with for authentication using [`PAM`](https://en.wikipedia.org/wiki/Pluggable_authentication_module) (requires `libpam`) |
| `shadow` | Builds with support for authentication using the shadow password database                                                       |

## Configuration

`mk` searches for rules defined in `/etc/mk.conf`, configured in the [`TOML`](https://toml.io/en/) format.

### Minimal configuration

```toml
[policies.default.permits]
all-targets = true

[groups]
wheel = "default"
```

### A more detailed example

```toml
# A policy defines how `mk` behaves
[policies]

# Definitions for a policy named "default"
[policies.default]

# Permitted actions
[policies.default.permits]
# Allow executing actions as all users
# Default: false
all-targets = false

# Users that this policy allows executing actions as
# Default: (empty)
targets = [
    "root"
]

# Runtime behavior
[policies.default.session]
# Allow users of this policy to execute actions without authentication
# Default: false
no-auth = false

# Inactive duration after which a user will need to be re-authenticated
# Default: -1 (no timeout) - the user will be re-authenticated each time
refresh = 5 # minutes

# A more restricted policy
[policies.restricted]

# Policy for groups
[groups]
wheel = "default"

# Policy to use for each user
[users]
# Dummy will use the restricted policy even if they
# are part of the wheel group
dummy = "restricted"
```

---

### Disclaimer

I am not an expert, and this project is still very early in development. Don't use this anywhere except for testing.

### License

`mk` and all its crates are distributed under the terms of the MIT license. See [LICENSE](LICENSE) for more information.

# mk (^∇^)-b


`mk` is a tool to run unix commands as another user, and a family of crates. It is similar to [`doas`](https://github.com/Duncaen/OpenDoas) or [`sudo`](https://github.com/sudo-project/sudo).

---

### Requirements

 - Rust's nightly toolchain (1.56 or higher)
 - A `C` compiler
 - Python 3.6+ for `x.py`

### `x.py`

`x.py` is a helper script to help build, test and install `mk`. Use `./x.py -h` to see all options.

### Examples

**Clean artifacts and rebuild `mk`**

```sh
$ ./x.py --clean --build release
```

**Install `mk`**

```sh
$ ./x.py --install
```

---


### Disclaimers

 - I am not an expert, and this project is still very early in development. Don't use this anywhere except for testing.
 - Linux support is targeted first.

### License

`mk` and all its crates are distributed under the terms of the MIT license. See [LICENSE](LICENSE) for more information.

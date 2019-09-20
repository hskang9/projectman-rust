![projectman-rust](https://i.imgur.com/Xwvpfrl.png)
# projectman-rust

Projectman meets Rust   
Projectman(in [crates.io](https://crates.io/crates/projectman) is the Rust port of [Projectman](https://github.com/saurabhdaware/projectman) by [Saurabh Daware](https://github.com/saurabhdaware)
. ProjectMan is a CLI which lets you add projects to favorites using command `pm add` and open them from anywhere you want using command `pm open`. Along with this there are also other commands like pm seteditor, pm remove, cd $(pm getpath) which we will see below.


# Changes

# Optimized memory

Total size of From the original(43MB) -> To Rust release binary(3.6MB)

# Compatibility 

This app is compatible with the original project's `setting.json` file.

# Installation

Make sure you add ~/.cargo/bin to your PATH to be able to run the installed binaries

```bash
echo 'export PATH="$HOME/.cargo/bin:$PATH"'  >> ~/.bash_profile
```

install CLI with cargo install
```Bash
cargo install projectman
```

# Future works
- [ ] wasmer runtime support
- [ ] add edit command 

# Credits

[Projectman](https://github.com/saurabhdaware/projectman) by [Saurabh Daware](https://github.com/saurabhdaware)


# Installation

### [Cargo](https://crates.io/crates/serie)

```
$ cargo install --locked serie
```

### [Arch Linux](https://archlinux.org/packages/extra/x86_64/serie/)

```
$ pacman -S serie
```

### [Homebrew](https://formulae.brew.sh/formula/serie)

```
$ brew install serie
```

or from [tap](https://github.com/lusingander/homebrew-tap/blob/master/serie.rb):

```
$ brew install lusingander/tap/serie
```

### [NetBSD](https://pkgsrc.se/devel/serie)

```
$ pkgin install serie
```

### Downloading binary

You can download pre-compiled binaries from [releases](https://github.com/lusingander/serie/releases).

### Build from source

If you want to check the latest development version, build from source:

```
$ git clone https://github.com/lusingander/serie.git
$ cd serie
$ cargo build --release # Unless it's a release build, it's very slow.
$ ./target/release/serie
```

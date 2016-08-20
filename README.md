liru [![Build Status](https://travis-ci.org/flugsio/liru.svg?branch=master)](https://travis-ci.org/flugsio/liru)
---------------

liru is a toy TUI client for the [lila](https://github.com/ornicar/lila) project running at [lichess.org](https://lichess.org)

It includes features like watching games, login to play in existing games, and showing various runtime errors for your debugging pleasure.

To build it you need at least rust 1.8.0, and some version of cargo for the dependencies.

```
cargo run --release
```

For autologin while developing, use something like this:

```
cargo build && echo "flugsio\n$(pass lichess.org)" | cargo run 2> stdout.log
```

I recommend [pass](https://www.passwordstore.org/) if you don't already have a password manager.


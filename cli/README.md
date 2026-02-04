# johnjeong (TUI)

Terminal edition of the site, built with `crossterm`.

It reads local content from the `part-of-my-brain` repo. Set `JOHNJEONG_CONTENT_DIR` if you want
to point at a custom location.

## Run locally

```bash
cargo run
```

Or specify content explicitly:

```bash
JOHNJEONG_CONTENT_DIR=../part-of-my-brain cargo run
```

## Build

```bash
cargo build --release
```

Binary will be at:

```
cli/target/release/johnjeong
```

## Key bindings

- `1-6` switch tabs
- `g` gallery tab
- `↑/↓` or `j/k` move selection
- `pgup/pgdn` scroll content
- `o` or `enter` open link
- `q` quit

## Homebrew distribution

1. Create a GitHub release from this repo.
2. Upload the source tarball (or use the auto-generated one).
3. Update the Homebrew formula at `homebrew/johnjeong.rb` with the release URL + SHA256.
4. Publish the formula in a tap repo (e.g. `johnjeong/homebrew-tap`).

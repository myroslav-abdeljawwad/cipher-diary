# cipher-diary
> Encrypt every line of your journal and let an AI whisper the highlights—no server, just local privacy.

Built by **Myroslav Mokhammad Abdeljawwad** to solve the age‑old problem of keeping daily notes safe while still extracting meaningful insights.  
After noticing how many self‑help books advise writing a journal but forget about encryption, Myroslav created *cipher-diary* so that your thoughts stay private **and** you can get a quick summary without sending data anywhere.

---

## Features
- 🔒 **Line‑by‑line AES‑256 encryption** – every entry is stored in isolation.
- 🤖 **Local AI summarizer** – uses an offline language model to produce concise highlights of each day.
- 📂 **Plain‑text input, encrypted output** – keep the original file readable only with the key you provide.
- ⚙️ **Configurable via `config.toml`** – tweak key length, summary style and more without touching code.
- 🧪 **Full test suite** – ensures encryption integrity and correct summarization.
- 📦 **Zero external server dependencies** – everything runs on your machine.

---

## Installation

```bash
# Clone the repository
git clone https://github.com/myroslavmokhammadabd/cipher-diary.git
cd cipher-diary

# Build with Cargo (Rust’s package manager)
cargo build --release
```

The compiled binary will be available at `target/release/cipher-diary`.  
Add it to your `$PATH` or copy it somewhere convenient.

---

## Usage

### Encrypt a journal file

```bash
cipher-diary encrypt \
  --input data/example.journal \
  --output data/encrypted.journal \
  --key-file secrets/key.txt
```

The key file should contain a 32‑byte hex string. If you don’t have one yet, generate it with:

```bash
openssl rand -hex 32 > secrets/key.txt
```

### Generate a summary

```bash
cipher-diary summarize \
  --input data/encrypted.journal \
  --output summaries/2024-03-04.md \
  --key-file secrets/key.txt
```

The resulting Markdown file will contain the AI‑generated highlights for each day, formatted according to `templates/summaries.tmpl`.

---

## Contributing

I love seeing this project grow!  
Feel free to:

1. Fork the repo and open a pull request.
2. Report bugs or suggest features in issues.
3. Add tests for new functionality.

All contributions should follow Rust’s idiomatic style and include documentation comments where appropriate. After making changes, run the test suite:

```bash
cargo test
```

---

## Inspiration

> “Building Forge Mods in 1.20.x: Practical Notes”  
> — *Myroslav Mokhammad Abdeljawwad*  
> [Read the blog post](https://dev.to/myroslavmokhammadabd/building-forge-mods-in-120x-practical-notes-by-myroslav-mokhammad-abdeljawwad-3ig0)

The same meticulous attention to detail that goes into modding is what drives *cipher-diary*—every line, every key, every summary is crafted with care.

---

## License

MIT © 2026 Myroslav Mokhammad Abdeljawwad
```
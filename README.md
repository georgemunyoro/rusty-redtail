
<br />
<p align="center">
  <a href="https://github.com/georgemunyoro/rusty-redtail">
     <img src="https://webtail.vercel.app/redtail.svg" width="200" height="200" />
  </a>

  <h3 align="center">Redtail Chess Engine</h3>

  <p align="center">
    <a href="https://lichess.org/@/redtail-zero"><strong>Check it out on Lichess</strong></a>
    <br />
    <a href="https://github.com/georgemunyoro/rusty-redtail/issues">Report Bug</a>
    ·
    <a href="https://github.com/georgemunyoro/rusty-redtail/issues">Request Feature</a>
  </p>
</p>

Redtail is a chess engine written in Rust. The engine employs various algorithms and strategies to make intelligent moves and offer a challenging opponent for chess enthusiasts. It may not be stockfish, but it's mine :)

Challenge redtail to a game on [lichess](https://lichess.org/@/redtail-zero) or play redtail in the browser with WASM: https://webtail.vercel.app/ (Note that the WASM version is usually a bit behind, and its strength is dependant on your hardware)

## Getting Started

To use redtail, follow these steps:

1. Clone the repository:

   ```
   git clone https://github.com/georgemunyoro/rusty-redtail.git
   ```

2. Build the project using Cargo:

   ```
   cd redtail
   cargo build --release
   ```

3. Run the engine:

   ```
   cargo run --release
   ```

4. Interact with the engine via a user interface or a chess GUI that supports the Universal Chess Interface (UCI) protocol.

## Contributing

Contributions to redtail are welcome! If you encounter any issues or have suggestions for improvements, please open an issue on the GitHub repository. Feel free to fork the project and submit pull requests as well.

## License

This project is licensed under the [MIT License](LICENSE). Feel free to use, modify, and distribute the code.

## Acknowledgements

- [Chess Programming Wiki](https://www.chessprogramming.org/)
- [Rust Programming Language](https://www.rust-lang.org/)
- [Bruce Moreland](https://web.archive.org/web/20071026090003/http://www.brucemo.com/compchess/programming/index.htm)
- [Stockfish](https://github.com/official-stockfish/Stockfish)

---

Enjoy playing chess with redtail!

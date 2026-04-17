# Postfix Interpreter in Rust

This is a simple implementation of a postfix, which is introduced in Design Concepts in Programming Languages, in Rust. 

## Usage

```bash
cargo run -- "(postfix 2 (mul sub) (1 nget mul) (4) nget swap exec swap exec)" "-10 2"
````

This will output following:

```
"(postfix 2 (mul sub) (1 nget mul) 4 nget swap exec swap exec)" "-10 2"
"(postfix 0 2 -10 (mul sub) (1 nget mul) 4 nget swap exec swap exec)" ""
"(postfix 0 2 -10 (mul sub) (1 nget mul) 2 swap exec swap exec)" ""
"(postfix 0 2 -10 (mul sub) 2 (1 nget mul) exec swap exec)" ""
"(postfix 0 2 -10 (mul sub) 2 1 nget mul swap exec)" ""
"(postfix 0 2 -10 (mul sub) 2 2 mul swap exec)" ""
"(postfix 0 2 -10 (mul sub) 4 swap exec)" ""
"(postfix 0 2 -10 4 (mul sub) exec)" ""
"(postfix 0 2 -10 4 mul sub)" ""
"(postfix 0 2 -40 sub)" ""
"(postfix 0 42)" ""
```

## File Structure
- `src/main.rs`: The main entry point.
- `src/parser.rs`: Postfix lexer and parser.
- `src/postfix.rs`: The postfix interpreter.
- `src/rewrite.rs`: The postfix interpreter with rewrite rules.
- `src/lib.rs`: Library for entry point.

## License
This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
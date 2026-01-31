<p align="center">
  <img src="kiro-logo.png" width="300" alt="Kiro Logo">
</p>

# Kiro Language Support for VS Code

This is the official Visual Studio Code extension for **Kiro**, a modern experimental programming language.

## ✨ Features

- **Syntax Highlighting**: Comprehensive coloring for Kiro's unique syntax, including:
  - Keywords (`fn`, `var`, `struct`, `import`, `pure`, `run`, `error`)
  - Control Flow (`if`, `loop`, `on/off`)
  - Types (`num`, `str`, `bool`, `void`, `adr`, `pipe`)
  - Operators (`ref`, `deref`, `!`, `push`, `at`)
  - Comments and Strings
- **Bracket Matching**: Automatic matching for `{ }`, `( )`, and `[ ]`.
- **Comment Toggling**: Support for `//` line comments.

## 📦 Installation

This extension is currently part of the Kiro language repository. To use it locally:

1. Open the `kiro-vscode/kiro` folder in VS Code.
2. Press `F5` to launch a new Extension Development Host window with Kiro support enabled.
3. Open any `.kiro` file to see the syntax highlighting in action.

## 🚀 Usage

Create a file ending in `.kiro` (e.g., `main.kiro`). The extension will automatically activate and provide syntax highlighting.

```kiro
fn main() -> void {
    print "Hello, Kiro!"

    var x = 10
    on (x > 5) {
        print "Greater than 5"
    }
}
```

## 🔗 Kiro Language

To learn more about the Kiro language, visit the [main repository](https://github.com/Start-0/kiro-lang).

## 📄 License

MIT

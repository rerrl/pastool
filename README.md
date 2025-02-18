# Pastool

Pass, with Frosting!

Pastool is a Rust-powered GUI wrapper for pass, making your password management even sweeter.

## Development

PRs are welcome, but please open an issue first to discuss your ideas, unless it's something trivial.

To run the app locally, you'll need to install the following dependencies:

- Node.js
- Rust

Then, run the following commands:

```bash
npm install
npm run tauri:dev
```

This will start the development server and open the app in your browser.

## Building

To build the app, run the following command:

```bash
npm run tauri:build
```

This will create a `dist` directory with the built app. Bundled builds can be found in th e`src-tauri/target/release` directory.

## Goals

- Rust based desktop password manager utilizing Password Store
- Make easy to use for someone new to pass (and password managers in general)
- Idea is to be able to recommend this to someone who isnt very techy and needs a password manager
- Make familiar for someone who is experienced with Pass specifically (power users can still find this useful in one way or another, though passmenu is superior for quick access)
- hotkey accessibility, task manager integration
- make search like dmenu search
- open source
- cross platform (windows, linux, macos)

To Add:

- [ ] git integration
- [ ] better theme
- better generate page
  - [ ] more consistent UI
  - [ ] add custom symbols to password generation
  - [ ] warn if overwrites existing file
  - [ ] Clear clipboard after copy
  - [ ] add password obfuscation in generated page (hide/show)
- better search page
  - [ ] better name listing (overflow and enabled horizontal scroll. Maybe a tree view?)
  - [ ] click password to copy or to view (edit?)

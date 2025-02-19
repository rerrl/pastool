# Pastool

Pass, with Frosting!

Pastool is a Rust-powered GUI wrapper for pass, making your password management even sweeter.

![image](https://github.com/user-attachments/assets/c4043590-e007-4095-86f5-8fee574141ac)
![image](https://github.com/user-attachments/assets/d147b337-a452-47a4-b5bf-c7b0a7f92c5a)


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

- [ ] icons
- [ ] git integration
- [ ] better themes
  - [ ] dark mode
  - [x] light mode
- better generate page
  - [ ] more consistent UI
  - [ ] add custom symbols to password generation
  - [ ] warn if overwrites existing file
  - [ ] Clear clipboard after copy
  - [ ] add password obfuscation in generated page (hide/show)
- better search page
  - [ ] better name listing (overflow and enabled horizontal scroll. Maybe a tree view?)

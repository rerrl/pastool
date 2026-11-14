# Pastool

Pass, with Frosting!

Pastool is a Rust-powered GUI wrapper for pass, making your password management even sweeter.

![image](https://github.com/user-attachments/assets/c4043590-e007-4095-86f5-8fee574141ac)
![image](https://github.com/user-attachments/assets/d147b337-a452-47a4-b5bf-c7b0a7f92c5a)

## Features

- Browse your pass store and copy an entry straight to the clipboard
- Copies only the password — the first line of the entry — not the metadata below it
- Clears the clipboard 45 seconds after a copy (skipped if something else has replaced it since)
- Generate new passwords, with a show/hide toggle
- Never overwrites an existing entry — generation refuses a name that's already taken

## First run

On first launch Pastool checks that it has everything it needs and walks you through the setup, one step at a time:

1. **Install check** — looks for `gpg` and `pass`, and shows the install command for your package manager if either is missing.
2. **GPG key** — creates a key for you (just a name and email as a label; nothing leaves your machine), or uses an existing one.
3. **Password store** — runs `pass init` with the chosen key, creating `~/.password-store`.

Once that's done the wizard gets out of the way and you land in the app.

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
  - [x] warn if overwrites existing file
  - [x] Clear clipboard after copy
  - [x] add password obfuscation in generated page (hide/show)
- better search page
  - [ ] better name listing (overflow and enabled horizontal scroll. Maybe a tree view?)

## Testing safely alongside an existing pass setup

Pastool works on your real `~/.password-store` and GPG keyring, so if you already use pass and just want a look around, move your store aside first:

```bash
mv ~/.password-store ~/.password-store.real
npm run tauri:dev
# ... poke around, including the first-run wizard ...
mv ~/.password-store.real ~/.password-store   # restore when you're done
```

Be aware the wizard creates a **real** GPG key in your keyring and runs `pass init` against it. To clean up a test key afterwards:

```bash
gpg --list-secret-keys                # find the key id you just made
gpg --delete-secret-keys <key-id>      # delete the secret half first
gpg --delete-keys <key-id>             # then the public half
```

Anything encrypted with that key becomes unreadable once it's gone, so only delete the test key.

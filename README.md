# vile

vile is a desktop editor for Vial keyboard layouts (`.vil` files). Edit keys and
layers, review the changes, then save the file.

## Download

Download the macOS Apple silicon, Linux x86_64, or Windows x64 app from the
[latest release](https://github.com/educorrea17/vile/releases/latest).

## Updates

vile can update itself. Automatic checks are off by default. To turn them on,
open the command menu (`⌘K` or `Ctrl+K`) and choose **Updates: turn on automatic
checks**. vile then checks about 10 seconds after launch and once a day,
downloads a new version in the background, and offers **Restart to update**.
Choose **Updates: turn off automatic checks** to stop. **Check for updates**
always works.

Each check sends a request to github.com. The request carries your IP address
and the updater's User-Agent. It sends no layout data.

- **macOS:** keep vile in Applications. A copy run from Downloads cannot update.
- **Linux, Debian package:** an update asks for an administrator password
  (pkexec).
- **Linux, AppImage:** an update replaces the AppImage file in place.
- **Windows:** the installer window appears briefly during an update.

Version 0.3.1 and older cannot update themselves. Install the release that adds
updates once by hand. Later versions update automatically.

## Run from source

Install Node.js 26.12.1 and Rust 1.90 or newer. Tauri also needs system build
dependencies for your operating system. See the [Tauri v2 prerequisites](https://v2.tauri.app/start/prerequisites/).

From the repository root, install the app dependencies and start the desktop app:

```sh
npx --yes npm@12.2.1 --prefix app ci
npx --yes npm@12.2.1 --prefix app run tauri -- dev --no-watch
```

To build the app for your current operating system, run:

```sh
npx --yes npm@12.2.1 --prefix app run tauri -- build --no-sign
```

`--no-sign` skips updater signing. Only the release workflow has the signing key.

## Edit a layout

Open a `.vil` file with **Open file…** (`⌘O` on macOS, `Ctrl+O` elsewhere). Select
keys on the board and choose a keycode, or press Enter to type one. Use the
layer controls to copy, move, or swap existing layers. The command menu (`⌘K` or
`Ctrl+K`) lists the available actions.

Edits stay in the app until you save. Review the diff, then choose **Save** or
**Save as…**. If you name keys, vile stores them in a neighboring file with
`.vile.json` appended to the full layout filename. For example,
`my-keymap.vil.vile.json` belongs beside `my-keymap.vil`. Keep both files when
you move or share the layout.

You can edit offline without a connected keyboard. After saving, load the `.vil`
in Vial with **File → Load saved layout**. If a keyboard is connected, vile reads
its identity and layout geometry. It does not flash firmware or write keyboard
settings.

## Command line

The repository includes a CLI for checking and inspecting layouts:

```sh
cargo run -q -p vile-cli -- check path/to/layout.vil
cargo run -q -p vile-cli -- show path/to/layout.vil --layer 0
```

File-changing commands print a diff and leave the input unchanged unless you
pass `--write` or `--output PATH`.

See [CHANGELOG.md](CHANGELOG.md) for release notes.

## License

vile is licensed under the GNU General Public License, version 3 or later
(GPL-3.0-or-later). See [LICENSE](LICENSE).

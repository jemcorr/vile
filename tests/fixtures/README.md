# Test fixtures

- `corne.vil`: a copy of `/Users/eduardocorrea/git/kb/config.vil`, the user's
  Corne v4.1 layout, taken when implementation started. Tests that use it only
  check properties that hold for any file Vial saved (byte-identical round trip,
  uid, shape, macro slots), so it may be refreshed with a newer copy.
- `corne_v32.vil`: `config.vil` at commit
  68884a21af18d55788c6afed058257bf346ddb18 of the kb repository ("config v32"),
  sha256 afd70085687d5f26b2cbb09e3633425772a25e5f2b5659dbb81641760deeb6c2.
  Tests assert concrete keys from this file. Never replace or edit it.

Both files are written by Vial (Python `json.dumps`) and must stay byte for
byte unchanged. Never write to the originals in /Users/eduardocorrea/git/kb.

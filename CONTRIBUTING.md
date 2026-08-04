<!--
SPDX-FileCopyrightText: 2025-2026 Spencer
SPDX-License-Identifier: AGPL-3.0-only
-->

### Contributing to Aletheia
Thanks for your interest in contributing to Aletheia. Whether you want to fix bugs, add features, improve the GameDB, or help with translations, contributions are welcome.

### GameDB
Add game save locations in `resources/gamedb.yaml`; entries must be alphabetical, and the file is automatically linted on pull requests. Include Linux paths if supported. Game titles are based on GOG names, but titles from Steam and itch.io are also accepted. The following placeholders can be used:

> Note: If the game is on Steam and/or GOG, you must include `store_ids` (see below) rather than relying on the local launcher name. This matters because Steam sometimes stores a game's name in the developer's language, such as Japanese or Korean, in the local appmanifest_<appid>.acf, regardless of your Steam client or system language, which would otherwise cause name matching to fail. For games not available on Steam or GOG (e.g. itch.io-only), name matching is used as normal since no store ID applies.

| Placeholder       | Description                                                                                   |
|-------------------|-----------------------------------------------------------------------------------------------|
| `{GameRoot}`      | Root directory of the game installation                                                       |
| `{AppData}`       | Roaming AppData folder on Windows and Application Support on MacOS                            |
| `{LocalAppData}`  | Local AppData folder on Windows                                                               |
| `{LocalLow}`      | LocalLow AppData folder on Windows                                                            |
| `{Documents}`     | User’s documents directory                                                                    |
| `{Home}`          | User’s home directory                                                                         |
| `{XDGConfig}`     | Linux XDG config directory                                                                    |
| `{XDGData}`       | Linux XDG data directory                                                                      |
| `{GOGAppData}`    | GOG application data directory                                                                |
| `{SteamID3}`      | Steam ID3                                                                                     |
| `{SteamID64}`     | Steam ID64                                                                                    |
| `{SteamUserData}` | Steam userdata directory for the configured Steam user                                        |

Example entry:
```yaml
Unleashed Recompiled:
  files:
    linux:
      - "{XDGConfig}/UnleashedRecomp/save/*"
    windows:
      - "{AppData}/UnleashedRecomp/save/*"
```

On Linux, `windows` paths are resolved relative to the Wine prefix rather than a real Windows filesystem, so you don't need to write separate entries to cover Proton/Wine.

#### Matching by store ID
An entry can optionally include `store_ids` with a Steam App ID and/or a GOG product ID. When present, Aletheia matches an installed game against these IDs before falling back to name matching, and renames the detected game to the entry's key.
This is the preferred way to handle games whose local launcher name doesn't match the entry's title, e.g. `appmanifest_4141950.acf` for "Kugayama Shiori's Death Diary" stores the name as 久我山栞の死様手帖 rather than the English title, so name matching alone wouldn't find it:

```yaml
Kugayama Shiori's Death Diary:
  files:
    windows:
      - "{SteamUserData}/4141950/remote/*"
  store_ids:
    steam: 4141950
```

Include both `store_ids.steam` and `store_ids.gog` if you know both. Verify IDs against the actual store page or installed game before submitting, since an incorrect ID will cause save data to be matched against the wrong entry.

### Translations
Translations are managed with [Weblate](https://weblate.org); you can contribute translations [here](https://hosted.weblate.org/projects/aletheia).

### AI Usage Policy
AI-generated or AI-assisted contributions are not accepted. This includes, but is not limited to, large language models (LLMs), code generators, and AI-based translation tools. All contributions (code, documentation, GameDB entries, and translations) must be written entirely by a human. Pull requests found to include AI-generated or AI-assisted content will be rejected.

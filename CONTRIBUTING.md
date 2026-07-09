<!--
SPDX-FileCopyrightText: 2025 Spencer
SPDX-License-Identifier: AGPL-3.0-only
-->

### Contributing to Aletheia
Thanks for your interest in contributing to Aletheia. Whether you want to fix bugs, add features, improve the GameDB, or help with translations, contributions are welcome.

### GameDB
Add game save locations in `resources/gamedb.yaml`; entries must be alphabetical, and the file is automatically linted on pull requests. Include Linux paths if supported. Game titles are based on GOG names, but titles from Steam and itch.io are also accepted. The following placeholders can be used:

> Note: Aletheia matches Steam games against the name in the local `appmanifest_<appid>.acf`, not the store page. This is usually the same as the English title, but Steam sometimes stores the name in the developer's own language, like Japanese or Korean, regardless of your Steam client or system language. If your entry isn't matching, check `steamapps/appmanifest_<appid>.acf` for the exact string Steam is using and key your entry off that.

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

On Linux, `windows` paths are resolved relative to the Wine prefix rather than a real Windows filesystem, so you don't need to write separate entries just to cover Proton/Wine.

### Translations
Translations are managed with [Weblate](https://weblate.org), you can contribute translations [here](https://hosted.weblate.org/projects/aletheia).

### AI Usage Policy
AI-generated or AI-assisted contributions are not accepted. This includes, but is not limited to, large language models (LLMs), code generators, and AI-based translation tools. All contributions (code, documentation, GameDB entries, and translations) must be written entirely by a human. Pull requests found to include AI-generated or AI-assisted content will be rejected.

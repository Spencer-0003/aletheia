<!--
SPDX-FileCopyrightText: 2025 Spencer
SPDX-License-Identifier: AGPL-3.0-only
-->

### Contributing to Aletheia
Thanks for your interest in contributing to Aletheia. Whether you want to fix bugs, add features, improve the GameDB, or help with translations, contributions are welcome.

### GameDB
Add game save locations in `resources/gamedb.yaml`; entries must be alphabetical, and the file is automatically linted on pull requests. Include Linux paths if supported. Game titles are based on GOG names, but titles from Steam and itch.io are also accepted. The following placeholders can be used:

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
| `{SteamUserData}` | Steam userdata directory                                                                      |

Example entry:
```yaml
Unleashed Recompiled:
  files:
    linux:
      - "{XDGConfig}/UnleashedRecomp/save/*"
    windows:
      - "{AppData}/UnleashedRecomp/save/*"
```

### Translations
Translations are managed with [Weblate](https://weblate.org), you can contribute translations [here](https://hosted.weblate.org/projects/aletheia).

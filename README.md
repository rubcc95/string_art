# String Art Application

Convert images into thread patterns for creating string art. It generates detailed instructions in text format and provides graphical previews of the resulting patterns.

More detailed instructions on how to use the application will be published in future versions.

## Language

The interface is available in English, Spanish, French, Portuguese, Italian,
German, Catalan, Galician and Basque. By default it follows the system
language, falling back to English when the system language is not supported.

The preference is stored in:

- Linux: `$XDG_CONFIG_HOME/string_art/config.json` (usually
  `~/.config/string_art/config.json`)
- macOS: `~/Library/Application Support/com.rubcc95.string_art/config.json`
- Windows: `%APPDATA%\rubcc95\string_art\config\config.json`
- Web: the `string_art.config` entry in `localStorage`

The file is optional. To force a language, create it with:

```json
{
  "version": 1,
  "language": "es"
}
```

Supported values are `"en"`, `"es"`, `"fr"`, `"pt"`, `"it"`, `"de"`, `"ca"`,
`"gl"`, `"eu"`, or `null`/missing for automatic detection. Regional tags such
as `"pt-BR"` or `"ca-ES"` are normalized to their base language. On native
builds, the `STRING_ART_LANG` environment variable overrides the file, which is
useful for tests and screenshots.

## Lastest Version Download Links

| Architecture | Installer | Portable |
|--------------|----------------|-----------------------|
| Windows x64  | [Download](https://github.com/rubcc95/string_art/releases/download/Windows/string_art_installer.exe) | [Download](https://github.com/rubcc95/string_art/releases/download/Windows/string_art_portable.exe) |
| macOS x64    | Not available  | Not available         |
| Linux x64    | Not available  | [Download](https://github.com/rubcc95/string_art/releases/download/Linux/string_art_portable) |


Older compiled versions of the gui app can be found [here](https://github.com/rubcc95/string_art/releases).
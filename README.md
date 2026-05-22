# Concierge

macOS system control MCP server — battery, clipboard, volume, text-to-speech, file operations, and wake prevention.

## Tools

| Tool | Description |
|------|-------------|
| `battery` | Get battery status (%) |
| `clipboard_read` | Read text from clipboard |
| `clipboard_write` | Write text to clipboard |
| `open` | Open file/URL with default app |
| `reveal` | Reveal file in Finder |
| `volume` | Get/set/mute/unmute system volume |
| `say` | Speak text aloud (TTS) |
| `wake` | Keep Mac awake (Amphetamine) |

## Prompts

- **system-status** — Returns battery, volume, and clipboard for summarization

## Resources

- `concierge://system/battery` — Current battery status
- `concierge://system/volume` — Current volume level

## Usage

Run with an MCP client that supports stdio transport:

```bash
cargo run
```

Install globally:

```bash
make install
```

## Requirements

- macOS
- [Amphetamine](https://apps.apple.com/app/id937984704) (for `wake` tool)

## Markdown PDF Export

Ask the Agent to export a Markdown file to PDF, with your header, footer, page numbers and logo.
Nothing else to install: Zed downloads the exporter and runs it with its own Node.

**Try it:** open a Markdown file and ask the Agent *"Export this file to PDF"*.

### Browser

The PDF is drawn by Google Chrome or Microsoft Edge, found automatically. No Chrome or Edge on
this computer? Ask the Agent *"Install the PDF browser"*: it downloads Chromium once (about 200 MB).

### Page design (optional)

The default look works as is. To choose the page style, fonts, header, footer and logo with a live
preview, ask the Agent *"Open Studio"*. Your choices are saved in the settings file shared with the
command line and other editors:

- macOS and Linux: `~/.config/markdown-pdf-export/settings.json`
- Windows: `%APPDATA%\markdown-pdf-export\settings.json`

### Settings below (all optional)

- `browser`: `chrome`, `msedge`, `chromium`, or the full path to a browser program.
- `settings_file`: path to another settings file, `~/...` works. Studio does not edit this file.
- `debug`: `true` to write a log to `markdown-pdf-debug.log` in the temp folder.

Something wrong? Ask the Agent *"Run the Markdown PDF doctor"*. Messages about updating the
exporter (lines starting with `[markdown-pdf]`) are not in Zed's log: to see them, quit Zed and
start it from a terminal with `zed --foreground`.
Documentation: <https://markdown-pdf.matinfo.tech>

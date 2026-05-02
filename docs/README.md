# Mago documentation builder

A standalone Rust builder for the static documentation site. `cargo run` writes
a flat tree of HTML to `dist/`. There is no dev server and no live reload, just
re-run the command after any change.

## Run locally

1. Install Node tooling for Pagefind (one-time):
   ```bash
   npm install
   ```
2. Build:
   ```bash
   cargo run -p mago-documentation
   ```

The output uses absolute asset paths (`/_assets/…`), so `file://` URLs do not
work. Serve `dist/` with PHP's built-in server (already on every Mago
contributor's machine):

```bash
cd dist && php -S 127.0.0.1:3031
```

Then open `http://127.0.0.1:3031/main/en/`.

## Add a page

1. Add a markdown file under `content/<lang>/…` with TOML front matter fenced by `+++`.
2. Provide `title`, `description`, `nav_order`, and `nav_section`.

## Add a language

1. Add the entry to `config.toml` under `[[languages]]`.
2. Create `content/<new-lang>/` with a mirrored page tree.
3. Add UI strings in `i18n/<new-lang>.toml`.

## Add a redirect

1. Append a `[[redirect]]` block to `redirects.toml`.
2. Set `from`, `to`, and whether query/hash must be preserved.
3. Rebuild and verify the generated stub at `dist/<from>/index.html`.

## Notes

- Static assets live under `static/` and are copied to `dist/_assets/`.
- The linter rules reference (`content/<lang>/tools/linter/rules.md`) is
  regenerated on every build by walking the in-process `mago-linter` registry.
- The playground template loads CodeMirror from a CDN via an import map; no
  bundle ships with the site.

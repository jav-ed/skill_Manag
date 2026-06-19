# Server Precompression Template

Use this template on the map server, not in the frontend repo, when static map assets need gzip, Brotli, and Zstandard sidecars.

The script compresses only files that already exist in the server asset root. That avoids the bad state where a `.pbf.br` or `.pbf.zst` sidecar exists without the matching base `.pbf` file.

## Copy To The Server

Copy [compress_Existing_Map_Static_Assets.sh](./compress_Existing_Map_Static_Assets.sh) to the server, for example:

```text
/home/javed/Fast/<handoff-folder>/server_Static_Compression/compress_Existing_Map_Static_Assets.sh
```

Run it against the actual static map asset root:

```bash
bash compress_Existing_Map_Static_Assets.sh /var/lib/maps/assets/basemaps
```

The server needs these commands:

```text
jq
gzip
brotli
zstd
```

If `brotli` is missing, install it before running the script if `.br` sidecars are required.

## What It Does

- Minifies existing `*.json` files in place, then creates `.gz`, `.br`, and `.zst` sidecars.
- Creates `.gz`, `.br`, and `.zst` sidecars for existing `*.pbf` files.
- Skips PNG and other already-compressed image files.
- Never guesses glyph ranges or font stacks.
- Never creates a sidecar without a matching base file.

Afterwards, configure Caddy's static asset route with:

```caddyfile
file_server {
	precompressed br zstd gzip
}
```

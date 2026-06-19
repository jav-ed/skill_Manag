import { mkdir, writeFile } from "node:fs/promises";
import { dirname } from "node:path";
import { brotliCompressSync, constants, gzipSync, zstdCompressSync } from "node:zlib";

// Copy this script into a project-level Code/Maps/ folder and adjust the import.
// This import path is written for that Code/Maps/ destination, not for this template folder.
// The imported function should return a MapLibre StyleSpecification object.
import { build_Project_Vector_Style } from "./project_Vector_Map_Style";

const attribution_Html = '<a href="https://www.openstreetmap.org/copyright" target="_blank" rel="noopener noreferrer">OpenStreetMap contributors</a>';

const style_Targets = [
  {
    file: "Scratch/G12_Handoff/map_Styles/<tenant>.public.json",
    tile_Url: "https://maps.example.com/t/<tenant>/<tileset>/{z}/{x}/{y}",
    glyphs_Url: "https://maps.example.com/assets/basemaps/fonts/{fontstack}/{range}.pbf",
    sprite_Url: "https://maps.example.com/assets/basemaps/sprites/v4/light",
  },
  {
    file: "Scratch/G12_Handoff/map_Styles/<tenant>.local-dev.json",
    tile_Url: "http://PRIVATE-MAP-IP/t/local-dev/<tileset>/{z}/{x}/{y}",
    glyphs_Url: "http://PRIVATE-MAP-IP/assets/basemaps/fonts/{fontstack}/{range}.pbf",
    sprite_Url: "http://PRIVATE-MAP-IP/assets/basemaps/sprites/v4/light",
  },
] as const;

async function write_Style_Bundle(file: string, style_Json: string) {
  const style_Buffer = Buffer.from(style_Json, "utf8");
  const gzip_Buffer = gzipSync(style_Buffer, { level: 9 });
  const brotli_Buffer = brotliCompressSync(style_Buffer, {
    params: {
      [constants.BROTLI_PARAM_QUALITY]: constants.BROTLI_MAX_QUALITY,
    },
  });
  const zstd_Buffer = zstdCompressSync(style_Buffer, {
    params: {
      [constants.ZSTD_c_compressionLevel]: 19,
    },
  });

  await mkdir(dirname(file), { recursive: true });
  await writeFile(file, style_Buffer);
  await writeFile(`${file}.gz`, gzip_Buffer);
  await writeFile(`${file}.br`, brotli_Buffer);
  await writeFile(`${file}.zst`, zstd_Buffer);

  console.log(
    `Wrote ${file} (${style_Buffer.byteLength} bytes, gzip ${gzip_Buffer.byteLength}, brotli ${brotli_Buffer.byteLength}, zstd ${zstd_Buffer.byteLength})`,
  );
}

for (const target of style_Targets) {
  const style = build_Project_Vector_Style({
    tile_Url: target.tile_Url,
    glyphs_Url: target.glyphs_Url,
    sprite_Url: target.sprite_Url,
    attribution_Html,
  });

  const style_Json = `${JSON.stringify(style)}\n`;

  await write_Style_Bundle(target.file, style_Json);
}

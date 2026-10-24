import { bounds } from "./interactions";
import type { BoardKeyDto, KeyCellDto, KeyKind, ViewModel } from "./types";

const UNIT = 60;
const GAP = 4;
const PADDING = 24;
const SCALE = 2;
const MAX_DIMENSION = 16384;
const MAX_PIXELS = 16_777_216;
type LayerExportView = Pick<ViewModel, "board" | "layers" | "names">;

export interface ExportTheme {
  background: string;
  foreground: string;
  muted: string;
  key: string;
  keyForeground: string;
  border: string;
  layer: string;
  modifier: string;
  macro: string;
  media: string;
  font: string;
}

export function readExportTheme(): ExportTheme {
  const style = getComputedStyle(document.documentElement);
  const color = (name: string) => style.getPropertyValue(name).trim();
  return {
    background: color("--bg"),
    foreground: color("--fg"),
    muted: color("--muted"),
    key: color("--key-bg"),
    keyForeground: color("--key-fg"),
    border: color("--key-border"),
    layer: color("--key-layer"),
    modifier: color("--key-mod"),
    macro: color("--key-macro"),
    media: color("--key-media"),
    font: style.fontFamily,
  };
}

/** Wrap the complete name, including names with no spaces, without truncation. */
function wrapTitle(title: string, width: number, measure: (text: string) => number): string[] {
  const lines: string[] = [];
  let line = "";
  for (const character of title) {
    if (line && measure(line + character) > width) {
      lines.push(line);
      line = "";
    }
    line += character;
  }
  if (line) lines.push(line);
  return lines;
}

export function planLayerSheet(view: LayerExportView, measure: (text: string) => number) {
  const keys = view.board?.keys;
  if (!keys?.length) throw new Error("Open a layout with keyboard geometry first");
  const layers = view.layers.flatMap((cells, index) =>
    cells.some((cell) => cell.code !== 0 && cell.code !== 1)
      ? [
          {
            index,
            cells,
            title: `Layer ${index}${view.names.layers[index] ? ` · ${view.names.layers[index]}` : ""}`,
          },
        ]
      : [],
  );
  if (!layers.length)
    throw new Error("No layers to export. Every layer contains only blank or transparent keys.");
  const box = bounds(keys);
  const boardWidth = Math.ceil(box.w * UNIT);
  const boardHeight = Math.ceil(box.h * UNIT);
  const tileWidth = Math.max(360, boardWidth + 2 * PADDING);
  const tiles = layers.map((layer) => ({
    ...layer,
    lines: wrapTitle(layer.title, tileWidth - 2 * PADDING, measure),
  }));
  const headingHeight = Math.max(...tiles.map((tile) => tile.lines.length)) * 26 + PADDING;
  const tileHeight = headingHeight + boardHeight + 2 * PADDING;
  // Prefer a compact, roughly square sheet while keeping layer order row-major.
  let columns = 1;
  let best = Infinity;
  for (let count = 1; count <= tiles.length; count++) {
    const width = count * tileWidth + (count + 1) * PADDING;
    const height =
      Math.ceil(tiles.length / count) * tileHeight +
      (Math.ceil(tiles.length / count) + 1) * PADDING;
    const score = Math.abs(Math.log(width / height));
    if (score < best) {
      best = score;
      columns = count;
    }
  }
  const rows = Math.ceil(tiles.length / columns);
  const width = (columns * tileWidth + (columns + 1) * PADDING) * SCALE;
  const height = (rows * tileHeight + (rows + 1) * PADDING) * SCALE;
  if (
    !Number.isFinite(width * height) ||
    width > MAX_DIMENSION ||
    height > MAX_DIMENSION ||
    width * height > MAX_PIXELS
  )
    throw new Error(
      `The layer sheet is too large to export (${width} × ${height} pixels). The limit is 16 megapixels and 16384 pixels per side.`,
    );
  return {
    keys,
    tiles: tiles.map((tile, position) => ({
      ...tile,
      x: PADDING + (position % columns) * (tileWidth + PADDING),
      y: PADDING + Math.floor(position / columns) * (tileHeight + PADDING),
    })),
    box,
    boardWidth,
    tileWidth,
    tileHeight,
    headingHeight,
    width,
    height,
  };
}

function keyColor(kind: KeyKind | undefined, theme: ExportTheme): string {
  switch (kind) {
    case "layer":
    case "layerTap":
    case "layerMod":
      return theme.layer;
    case "modifier":
    case "modTap":
    case "mods":
    case "oneShotMod":
      return theme.modifier;
    case "macro":
    case "tapDance":
      return theme.macro;
    case "media":
    case "mouse":
      return theme.media;
    default:
      return theme.key;
  }
}

function drawKey(
  ctx: CanvasRenderingContext2D,
  key: BoardKeyDto,
  cell: KeyCellDto | undefined,
  theme: ExportTheme,
): void {
  ctx.save();
  ctx.translate((key.rx ?? 0) * UNIT, (key.ry ?? 0) * UNIT);
  ctx.rotate(((key.r ?? 0) * Math.PI) / 180);
  ctx.translate(-(key.rx ?? 0) * UNIT, -(key.ry ?? 0) * UNIT);
  ctx.fillStyle = keyColor(cell?.kind, theme);
  ctx.strokeStyle = theme.border;
  ctx.lineWidth = 1.5;
  const rectangles = [
    { x: 0, y: 0, w: key.w, h: key.h },
    ...(key.secondary ? [key.secondary] : []),
  ];
  for (const rect of rectangles) {
    ctx.beginPath();
    ctx.roundRect(
      (key.x + rect.x) * UNIT + GAP,
      (key.y + rect.y) * UNIT + GAP,
      Math.max(1, rect.w * UNIT - GAP),
      Math.max(1, rect.h * UNIT - GAP),
      7,
    );
    ctx.fill();
    ctx.stroke();
  }
  const hold = cell?.kind === "modTap" || cell?.kind === "layerTap";
  const lines = (cell?.label ?? "").split("\n");
  if (hold) lines.reverse();
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  const lineHeight = Math.min(17, (key.h * UNIT - 12) / lines.length);
  lines.forEach((line, index) => {
    const small = hold && index === 0;
    ctx.font = `${small ? Math.min(10, lineHeight) : Math.min(16, lineHeight)}px ${theme.font}`;
    ctx.fillStyle = small || cell?.kind === "trns" ? theme.muted : theme.keyForeground;
    ctx.fillText(
      line,
      (key.x + key.w / 2) * UNIT + GAP / 2,
      (key.y + key.h / 2) * UNIT + GAP / 2 + (index - (lines.length - 1) / 2) * lineHeight,
      Math.max(1, key.w * UNIT - 14),
    );
  });
  ctx.restore();
}

/** Render from the complete model, independent of editor selection and viewport. */
export async function renderLayerSheet(
  view: LayerExportView,
  theme = readExportTheme(),
): Promise<{ png: Uint8Array; layerCount: number }> {
  await document.fonts?.ready;
  const canvas = document.createElement("canvas");
  try {
    const ctx = canvas.getContext("2d");
    if (!ctx) throw new Error("This web view cannot render a PNG export");
    ctx.font = `600 20px ${theme.font}`;
    const plan = planLayerSheet(view, (text) => ctx.measureText(text).width);
    canvas.width = plan.width;
    canvas.height = plan.height;
    ctx.scale(SCALE, SCALE);
    ctx.fillStyle = theme.background;
    ctx.fillRect(0, 0, plan.width / SCALE, plan.height / SCALE);
    for (const tile of plan.tiles) {
      ctx.strokeStyle = theme.border;
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.roundRect(tile.x, tile.y, plan.tileWidth, plan.tileHeight, 12);
      ctx.stroke();
      ctx.font = `600 20px ${theme.font}`;
      ctx.fillStyle = theme.foreground;
      ctx.textAlign = "left";
      ctx.textBaseline = "top";
      tile.lines.forEach((line, index) =>
        ctx.fillText(line, tile.x + PADDING, tile.y + PADDING + index * 26),
      );
      ctx.save();
      ctx.translate(
        tile.x + (plan.tileWidth - plan.boardWidth) / 2 - plan.box.x * UNIT,
        tile.y + plan.headingHeight + PADDING - plan.box.y * UNIT,
      );
      const cells = new Map(tile.cells.map((cell) => [cell.id, cell]));
      for (const key of plan.keys) drawKey(ctx, key, cells.get(key.id), theme);
      ctx.restore();
    }
    const blob = await new Promise<Blob>((resolve, reject) => {
      canvas.toBlob(
        (result) =>
          result ? resolve(result) : reject(new Error("The web view could not encode the PNG")),
        "image/png",
      );
    });
    return { png: new Uint8Array(await blob.arrayBuffer()), layerCount: plan.tiles.length };
  } finally {
    // Release the full-resolution bitmap after encoding or failure.
    canvas.width = canvas.height = 0;
  }
}

import { bounds } from "./interactions";
import { MOD_BITS, hasBit } from "./bits";
import { keycapKind } from "../components/Keycap.svelte";
import type { BoardKeyDto, KeyCellDto, KeycodeView, ViewModel } from "./types";

const UNIT = 60;
const GAP = 4;
const PADDING = 24;
const SCALE = 2;
const MAX_DIMENSION = 16384;
const MAX_PIXELS = 16_777_216;

// Feature cards reuse the editor keycap and spacing values (--space-1..5 = 4/8/12/16/24px).
const CAP = 44;
const CAP_GAP = 8;
const CARD_GAP = 12;
const CARD_PAD = 12;
const CARD_MIN = 320;
const CARD_RADIUS = 12;
const HEADING_LINE = 26;
const HEADING_GAP = 12;
const TITLE_LINE = 20;
const BODY_LINE = 18;
const BLOCK_GAP = 10;
const CHIP_PAD = 10;
const CHIP_PAD_Y = 5;
const CHIP_LINE = 16;
const CHIP_RADIUS = 8;
const CAPTION_LINE = 14;
const CAPTION_GAP = 4;
const ARROW_WIDTH = 24;

const FONT = {
  heading: "600 20px",
  title: "600 15px",
  body: "13px",
  caption: "11px",
  arrow: "16px",
} as const;

type LayerExportView = Pick<
  ViewModel,
  | "board"
  | "layers"
  | "names"
  | "keycodes"
  | "layerCount"
  | "combos"
  | "macros"
  | "tapDances"
  | "keyOverrides"
  | "altRepeatKeys"
>;
/** The advance width of text drawn in a CSS font shorthand such as "600 20px". */
export type Measure = (text: string, font: string) => number;
export type FeatureName = "combo" | "macro" | "tapDance" | "keyOverride" | "altRepeat";

export interface ExportTheme {
  background: string;
  foreground: string;
  muted: string;
  no: string;
  transparent: string;
  key: string;
  keyForeground: string;
  border: string;
  layer: string;
  modifier: string;
  macro: string;
  media: string;
  font: string;
}

// Blends two #rrggbb colors; t is the share of `a`. Mirrors --key-trns-bg in canvas.css.
export function mixHex(a: string, b: string, t: number): string {
  const parse = (hex: string): [number, number, number] => [
    parseInt(hex.slice(1, 3), 16),
    parseInt(hex.slice(3, 5), 16),
    parseInt(hex.slice(5, 7), 16),
  ];
  const [ra, ga, ba] = parse(a);
  const [rb, gb, bb] = parse(b);
  const channel = (x: number, y: number) => Math.round(x * t + y * (1 - t));
  const hex = (n: number) => n.toString(16).padStart(2, "0");
  return `#${hex(channel(ra, rb))}${hex(channel(ga, gb))}${hex(channel(ba, bb))}`;
}

export function readExportTheme(): ExportTheme {
  const style = getComputedStyle(document.documentElement);
  const color = (name: string) => style.getPropertyValue(name).trim();
  return {
    background: color("--bg"),
    foreground: color("--fg"),
    muted: color("--muted"),
    no: mixHex(color("--key-bg"), color("--bg"), 0.35),
    transparent: mixHex(color("--key-bg"), color("--bg"), 0.675),
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

type Part =
  | { type: "cap"; x: number; y: number; kind: string; label: string }
  | { type: "chip"; x: number; y: number; w: number; h: number; lines: string[] }
  | {
      type: "text";
      x: number;
      y: number;
      text: string;
      font: string;
      tone: "fg" | "muted";
      align: "left" | "center";
    };

interface Item {
  w: number;
  h: number;
  parts: Part[];
}

export interface Block {
  /** Top edge relative to the card. */
  y: number;
  h: number;
  /** Positions relative to the card content origin, inside CARD_PAD. */
  parts: Part[];
}

export interface Entry {
  feature: FeatureName;
  index: number;
  title: string[];
  blocks: Block[];
  x: number;
  y: number;
  w: number;
  h: number;
}

export interface Section {
  feature: FeatureName;
  heading: string;
  y: number;
  height: number;
  entries: Entry[];
}

interface Content {
  feature: FeatureName;
  index: number;
  title: string;
  blocks: { parts: Part[]; h: number }[];
}

const SECTION_HEADINGS: Record<FeatureName, string> = {
  combo: "Combos",
  macro: "Macros",
  tapDance: "Tap dances",
  keyOverride: "Key overrides",
  altRepeat: "Alt repeat",
};

const ENTRY_LABELS: Record<FeatureName, string> = {
  combo: "Combo",
  macro: "Macro",
  tapDance: "Tap dance",
  keyOverride: "Key override",
  altRepeat: "Alt repeat",
};

const MOD_LABELS: Record<string, string> = {
  LCTL: "L Ctrl",
  LSFT: "L Shift",
  LALT: "L Alt",
  LGUI: "L Gui",
  RCTL: "R Ctrl",
  RSFT: "R Shift",
  RALT: "R Alt",
  RGUI: "R Gui",
};

function shift(part: Part, dx: number, dy: number): Part {
  return { ...part, x: part.x + dx, y: part.y + dy };
}

/** Places items left to right and wraps them at width. Parts are relative to the block origin. */
function flow(items: Item[], width: number): { parts: Part[]; h: number } {
  const parts: Part[] = [];
  let x = 0;
  let y = 0;
  let rowH = 0;
  for (const item of items) {
    if (x > 0 && x + item.w > width) {
      x = 0;
      y += rowH + CAP_GAP;
      rowH = 0;
    }
    for (const part of item.parts) parts.push(shift(part, x, y));
    x += item.w + CAP_GAP;
    rowH = Math.max(rowH, item.h);
  }
  return { parts, h: items.length ? y + rowH : 0 };
}

function capItem(keycode: KeycodeView | undefined, fallback: string): Item {
  return {
    w: CAP,
    h: CAP,
    parts: [
      {
        type: "cap",
        x: 0,
        y: 0,
        kind: keycode ? keycapKind(keycode.code, keycode.category) : "other",
        label: keycode?.label || fallback,
      },
    ],
  };
}

function arrowItem(): Item {
  return {
    w: ARROW_WIDTH,
    h: CAP,
    parts: [
      {
        type: "text",
        x: ARROW_WIDTH / 2,
        y: (CAP - 18) / 2,
        text: "→",
        font: FONT.arrow,
        tone: "muted",
        align: "center",
      },
    ],
  };
}

/** A keycap with a caption above it, as used for each tap dance action. */
function captionedCapItem(
  caption: string,
  keycode: KeycodeView | undefined,
  fallback: string,
  measure: Measure,
): Item {
  const w = Math.max(CAP, Math.ceil(measure(caption, FONT.caption)));
  const cap = capItem(keycode, fallback);
  return {
    w,
    h: CAPTION_LINE + CAPTION_GAP + CAP,
    parts: [
      { type: "text", x: w / 2, y: 0, text: caption, font: FONT.caption, tone: "muted", align: "center" },
      ...cap.parts.map((part) => shift(part, (w - CAP) / 2, CAPTION_LINE + CAPTION_GAP)),
    ],
  };
}

function chipItem(text: string, width: number, measure: Measure): Item {
  const measureBody = (value: string) => measure(value, FONT.body);
  const lines = wrapTitle(text, width - 2 * CHIP_PAD, measureBody);
  const widest = Math.max(...lines.map(measureBody));
  const w = Math.min(width, Math.ceil(widest) + 2 * CHIP_PAD);
  const h = lines.length * CHIP_LINE + 2 * CHIP_PAD_Y;
  return { w, h, parts: [{ type: "chip", x: 0, y: 0, w, h, lines }] };
}

function textBlock(text: string, width: number, measure: Measure): { parts: Part[]; h: number } {
  const lines = wrapTitle(text, width, (value) => measure(value, FONT.body));
  return {
    parts: lines.map((line, i) => ({
      type: "text" as const,
      x: 0,
      y: i * BODY_LINE,
      text: line,
      font: FONT.body,
      tone: "muted" as const,
      align: "left" as const,
    })),
    h: lines.length * BODY_LINE,
  };
}

function modLabels(mask: number): string {
  const names = MOD_BITS.filter((bit) => hasBit(mask, bit.bit)).map(
    (bit) => MOD_LABELS[bit.name] ?? bit.name,
  );
  return names.length ? names.join(", ") : "none";
}

/** Entry content for every feature with at least one set slot, in editor order. */
function featureContents(view: LayerExportView, measure: Measure, width: number): Content[] {
  const contents: Content[] = [];
  const keycode = (path: string) => view.keycodes[path];
  const title = (feature: FeatureName, index: number, name?: string) =>
    `${ENTRY_LABELS[feature]} ${index}${name ? ` · ${name}` : ""}`;

  for (const combo of view.combos) {
    const triggers = combo.keys.flatMap((key, i) =>
      key === "KC_NO" ? [] : [capItem(keycode(`combo[${combo.index}][${i}]`), key)],
    );
    if (!triggers.length && combo.output === "KC_NO") continue;
    const items = [...triggers];
    if (combo.output !== "KC_NO") {
      items.push(arrowItem(), capItem(keycode(`combo[${combo.index}][4]`), combo.outputLabel || combo.output));
    }
    contents.push({
      feature: "combo",
      index: combo.index,
      title: title("combo", combo.index, view.names.combos[combo.index]),
      blocks: [flow(items, width)],
    });
  }

  for (const macro of view.macros) {
    if (!macro.actions.length) continue;
    const items: Item[] = [];
    macro.actions.forEach((action, i) => {
      if (action.kind === "text") items.push(chipItem(`"${action.text}"`, width, measure));
      else if (action.kind === "delay") items.push(chipItem(`${action.delay} ms`, width, measure));
      else {
        const labels = action.keys.flatMap((key, k) =>
          key === "KC_NO" ? [] : [keycode(`macro[${macro.index}][${i}][${k + 1}]`)?.label || key],
        );
        if (!labels.length) return;
        const prefix = action.kind === "down" ? "↓" : action.kind === "up" ? "↑" : "";
        items.push(chipItem(prefix + labels.join(" + "), width, measure));
      }
    });
    if (!items.length) continue;
    contents.push({
      feature: "macro",
      index: macro.index,
      title: title("macro", macro.index, view.names.macros[macro.index]),
      blocks: [flow(items, width)],
    });
  }

  const tapNames = ["Tap", "Hold", "Double tap", "Tap and hold"];
  for (const dance of view.tapDances) {
    const keys = [dance.onTap, dance.onHold, dance.onDoubleTap, dance.onTapHold];
    if (keys.every((key) => key === "KC_NO")) continue;
    const items = keys.flatMap((key, j) =>
      key === "KC_NO"
        ? []
        : [captionedCapItem(tapNames[j]!, keycode(`tap_dance[${dance.index}][${j}]`), key, measure)],
    );
    contents.push({
      feature: "tapDance",
      index: dance.index,
      title: title("tapDance", dance.index),
      blocks: [flow(items, width), textBlock(`Tapping term ${dance.tappingTerm} ms`, width, measure)],
    });
  }

  // The editor's layer checkboxes cover at most 16 layers, so "all layers" means every one of those.
  const layerIndexes = Array.from({ length: Math.min(view.layerCount, 16) }, (_, i) => i);
  for (const override of view.keyOverrides) {
    if (override.trigger === "KC_NO" && override.replacement === "KC_NO") continue;
    const items: Item[] = [];
    if (override.trigger !== "KC_NO") {
      items.push(capItem(keycode(`key_override[${override.index}].trigger`), override.trigger));
    }
    items.push(arrowItem());
    if (override.replacement !== "KC_NO") {
      items.push(capItem(keycode(`key_override[${override.index}].replacement`), override.replacement));
    }
    const blocks = [flow(items, width)];
    if (override.triggerMods !== 0) {
      blocks.push(textBlock(`Trigger mods: ${modLabels(override.triggerMods)}`, width, measure));
    }
    if (!layerIndexes.every((i) => hasBit(override.layers, i))) {
      const active = layerIndexes.filter((i) => hasBit(override.layers, i)).map((i) => `L${i}`);
      blocks.push(textBlock(`Layers: ${active.length ? active.join(", ") : "none"}`, width, measure));
    }
    contents.push({
      feature: "keyOverride",
      index: override.index,
      title: title("keyOverride", override.index),
      blocks,
    });
  }

  for (const repeat of view.altRepeatKeys) {
    const set =
      repeat.keycode !== "KC_NO" ||
      repeat.altKeycode !== "KC_NO" ||
      repeat.allowedMods !== 0 ||
      repeat.options !== 0;
    if (!set) continue;
    const items: Item[] = [];
    if (repeat.keycode !== "KC_NO") {
      items.push(capItem(keycode(`alt_repeat_key[${repeat.index}].keycode`), repeat.keycode));
    }
    items.push(arrowItem());
    if (repeat.altKeycode !== "KC_NO") {
      items.push(capItem(keycode(`alt_repeat_key[${repeat.index}].alt_keycode`), repeat.altKeycode));
    }
    const blocks = [flow(items, width)];
    // All eight modifier bits set means every modifier is allowed, so no line is needed.
    if ((repeat.allowedMods & 0xff) !== 0xff) {
      blocks.push(textBlock(`Allowed mods: ${modLabels(repeat.allowedMods)}`, width, measure));
    }
    contents.push({
      feature: "altRepeat",
      index: repeat.index,
      title: title("altRepeat", repeat.index),
      blocks,
    });
  }
  return contents;
}

const FEATURE_ORDER: FeatureName[] = ["combo", "macro", "tapDance", "keyOverride", "altRepeat"];

/**
 * One section per feature below the layer grid. Cards fill the layer grid width with the same
 * outer padding, and the layer tiles above keep their size and position.
 */
function layoutFeatures(
  view: LayerExportView,
  measure: Measure,
  sheetWidth: number,
  top: number,
): { sections: Section[]; bottom: number } {
  const inner = sheetWidth - 2 * PADDING;
  const columns = Math.max(1, Math.floor((inner + CARD_GAP) / (CARD_MIN + CARD_GAP)));
  const cardWidth = (inner - (columns - 1) * CARD_GAP) / columns;
  const contentWidth = cardWidth - 2 * CARD_PAD;
  const measureTitle = (text: string) => measure(text, FONT.title);
  const contents = featureContents(view, measure, contentWidth);
  const sections: Section[] = [];
  let y = top;
  for (const feature of FEATURE_ORDER) {
    const cards = contents.filter((content) => content.feature === feature);
    if (!cards.length) continue;
    const entries: Entry[] = [];
    let rowTop = y + HEADING_LINE + HEADING_GAP;
    for (let start = 0; start < cards.length; start += columns) {
      const row: Entry[] = cards.slice(start, start + columns).map((card, column) => {
        const titleLines = wrapTitle(card.title, contentWidth, measureTitle);
        let cursor = CARD_PAD + titleLines.length * TITLE_LINE + BLOCK_GAP;
        const blocks: Block[] = card.blocks.map((block, i) => {
          const placed: Block = { y: cursor, h: block.h, parts: block.parts };
          cursor += block.h + (i < card.blocks.length - 1 ? BLOCK_GAP : 0);
          return placed;
        });
        return {
          feature,
          index: card.index,
          title: titleLines,
          blocks,
          x: PADDING + column * (cardWidth + CARD_GAP),
          y: rowTop,
          w: cardWidth,
          h: cursor + CARD_PAD,
        };
      });
      // Cards in one row share a height so their borders line up.
      const rowHeight = Math.max(...row.map((entry) => entry.h));
      for (const entry of row) entry.h = rowHeight;
      entries.push(...row);
      rowTop += rowHeight + CARD_GAP;
    }
    const gridBottom = rowTop - CARD_GAP;
    sections.push({
      feature,
      heading: SECTION_HEADINGS[feature],
      y,
      height: gridBottom + PADDING - y,
      entries,
    });
    y = gridBottom + PADDING;
  }
  return { sections, bottom: y };
}

function keyColor(kind: string | undefined, theme: ExportTheme): string {
  switch (kind) {
    case "no":
      return theme.no;
    case "trns":
      return theme.transparent;
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

export function planLayerSheet(view: LayerExportView, measure: Measure) {
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
  const measureHeading = (text: string) => measure(text, FONT.heading);
  const tiles = layers.map((layer) => ({
    ...layer,
    lines: wrapTitle(layer.title, tileWidth - 2 * PADDING, measureHeading),
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
  const sheetWidth = columns * tileWidth + (columns + 1) * PADDING;
  const layerBottom = rows * tileHeight + (rows + 1) * PADDING;
  const { sections, bottom } = layoutFeatures(view, measure, sheetWidth, layerBottom);
  const width = sheetWidth * SCALE;
  const height = bottom * SCALE;
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
    sections,
    box,
    boardWidth,
    tileWidth,
    tileHeight,
    headingHeight,
    width,
    height,
  };
}

function drawCap(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  kind: string,
  label: string,
  theme: ExportTheme,
): void {
  ctx.fillStyle = keyColor(kind, theme);
  ctx.strokeStyle = theme.border;
  ctx.lineWidth = 1.5;
  ctx.beginPath();
  ctx.roundRect(x + 1, y + 1, CAP - 2, CAP - 2, 7);
  ctx.fill();
  ctx.stroke();
  const hold = kind === "modTap" || kind === "layerTap";
  const lines = label.split("\n");
  if (hold) lines.reverse();
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  const lineHeight = Math.min(16, (CAP - 12) / lines.length);
  lines.forEach((line, index) => {
    const small = hold && index === 0;
    ctx.font = `${small ? 10 : 14}px ${theme.font}`;
    ctx.fillStyle = small ? theme.muted : theme.keyForeground;
    ctx.fillText(
      line,
      x + CAP / 2,
      y + CAP / 2 + (index - (lines.length - 1) / 2) * lineHeight,
      CAP - 8,
    );
  });
}

function drawPart(
  ctx: CanvasRenderingContext2D,
  part: Part,
  ox: number,
  oy: number,
  theme: ExportTheme,
): void {
  if (part.type === "cap") {
    drawCap(ctx, ox + part.x, oy + part.y, part.kind, part.label, theme);
  } else if (part.type === "chip") {
    ctx.fillStyle = theme.key;
    ctx.strokeStyle = theme.border;
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.roundRect(ox + part.x, oy + part.y, part.w, part.h, CHIP_RADIUS);
    ctx.fill();
    ctx.stroke();
    ctx.font = `${FONT.body} ${theme.font}`;
    ctx.fillStyle = theme.keyForeground;
    ctx.textAlign = "center";
    ctx.textBaseline = "top";
    part.lines.forEach((line, index) =>
      ctx.fillText(line, ox + part.x + part.w / 2, oy + part.y + CHIP_PAD_Y + index * CHIP_LINE),
    );
  } else {
    ctx.font = `${part.font} ${theme.font}`;
    ctx.fillStyle = part.tone === "muted" ? theme.muted : theme.foreground;
    ctx.textAlign = part.align;
    ctx.textBaseline = "top";
    ctx.fillText(part.text, ox + part.x, oy + part.y);
  }
}

function drawFeatures(ctx: CanvasRenderingContext2D, sections: Section[], theme: ExportTheme): void {
  for (const section of sections) {
    ctx.font = `${FONT.heading} ${theme.font}`;
    ctx.fillStyle = theme.foreground;
    ctx.textAlign = "left";
    ctx.textBaseline = "top";
    ctx.fillText(section.heading, PADDING, section.y);
    for (const entry of section.entries) {
      ctx.strokeStyle = theme.border;
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.roundRect(entry.x, entry.y, entry.w, entry.h, CARD_RADIUS);
      ctx.stroke();
      ctx.font = `${FONT.title} ${theme.font}`;
      ctx.fillStyle = theme.foreground;
      ctx.textAlign = "left";
      ctx.textBaseline = "top";
      entry.title.forEach((line, index) =>
        ctx.fillText(line, entry.x + CARD_PAD, entry.y + CARD_PAD + index * TITLE_LINE),
      );
      for (const block of entry.blocks)
        for (const part of block.parts)
          drawPart(ctx, part, entry.x + CARD_PAD, entry.y + block.y, theme);
    }
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
    ctx.fillStyle = small ? theme.muted : theme.keyForeground;
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
    const measure: Measure = (text, font) => {
      ctx.font = `${font} ${theme.font}`;
      return ctx.measureText(text).width;
    };
    const plan = planLayerSheet(view, measure);
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
      ctx.font = `${FONT.heading} ${theme.font}`;
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
    drawFeatures(ctx, plan.sections, theme);
    const blob = await new Promise<Blob>((resolve, reject) => {
      canvas.toBlob(
        (result) =>
          result ? resolve(result) : reject(new Error("The web view could not encode the PNG")),
        "image/png",
      );
    });
    return {
      png: new Uint8Array(await blob.arrayBuffer()),
      layerCount: plan.tiles.length,
    };
  } finally {
    // Release the full-resolution bitmap after encoding or failure.
    canvas.width = canvas.height = 0;
  }
}

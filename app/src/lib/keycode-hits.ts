/**
 * The label to show beside a keycode hit. A label that only repeats the code's name
 * (KC_UNDO -> "UNDO", or any prefix-stripped form of the name) adds nothing, so it is omitted.
 */
export function hitLabel(name: string, label: string): string {
  const text = label.replace(/\n/g, " ").trim();
  if (text === "") return "";
  const normalize = (value: string): string => value.toUpperCase().replace(/[^A-Z0-9]/g, "");
  const target = normalize(text);
  const parts = name.split("_");
  for (let i = 0; i < parts.length; i++) {
    if (normalize(parts.slice(i).join("_")) === target) return "";
  }
  return text;
}

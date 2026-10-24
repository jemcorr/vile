import type { MacroActionDto } from "./types";
export function moveAction(actions: MacroActionDto[], from: number, to: number): MacroActionDto[] {
  if (from < 0 || to < 0 || from >= actions.length || to >= actions.length) return actions;
  const next = [...actions];
  const [action] = next.splice(from, 1);
  next.splice(to, 0, action!);
  return next;
}
export function macroSummary(actions: MacroActionDto[]): string {
  return (
    actions
      .map((a) =>
        a.kind === "text"
          ? `Text: ${a.text}`
          : a.kind === "delay"
            ? `${a.delay} ms`
            : `${a.kind}: ${a.keys.join(" + ")}`,
      )
      .join("; ") || "Empty"
  );
}

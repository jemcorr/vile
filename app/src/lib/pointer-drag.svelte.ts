export type DragPayload =
  | { kind: "keycode"; wire: string; code: number | null }
  | { kind: "macro"; slot: number; index: number }
  | {
      kind: "featureSlot";
      feature: "combo" | "macro" | "tapDance" | "keyOverride" | "altRepeat";
      index: number;
    };
export interface PointerDrop {
  payload: DragPayload;
  clientX: number;
  clientY: number;
  altKey?: boolean;
  featureTarget?: { feature: string; index: number; empty: boolean } | null;
}
export const pointerDrag = $state<{
  current: (PointerDrop & { label: string }) | null;
}>({ current: null });
let cancelActive: (() => void) | undefined;
let suppress = false;
export function suppressDragClick(): boolean {
  return suppress;
}

/** Keep in-app dragging out of the operating system's native drag loop. */
export function startPointerDrag(event: PointerEvent, payload: DragPayload, label: string): void {
  if (event.button !== 0) return;
  cancelActive?.();
  const source = event.currentTarget as Element;
  const id = event.pointerId;
  const startX = event.clientX,
    startY = event.clientY;
  const listeners = new AbortController();
  let moved = false;
  const finish = () => {
    listeners.abort();
    pointerDrag.current = null;
    cancelActive = undefined;
    if (source.hasPointerCapture(id)) source.releasePointerCapture(id);
    if (moved) {
      suppress = true;
      setTimeout(() => (suppress = false), 0);
    }
  };
  cancelActive = finish;
  source.setPointerCapture(id);
  window.addEventListener(
    "pointermove",
    (e) => {
      if (e.pointerId !== id) return;
      moved ||= Math.hypot(e.clientX - startX, e.clientY - startY) > 6;
      if (moved) {
        const candidate = document
          .elementFromPoint(e.clientX, e.clientY)
          ?.closest<HTMLElement>("[data-feature-slot-kind][data-feature-slot-index]");
        const feature = candidate?.dataset.featureSlotKind;
        const index = Number(candidate?.dataset.featureSlotIndex);
        const featureTarget =
          payload.kind === "featureSlot" &&
          feature === payload.feature &&
          candidate?.dataset.featureSlotReady !== "false" &&
          Number.isInteger(index) &&
          index !== payload.index
            ? { feature, index, empty: candidate?.dataset.state === "empty" }
            : null;
        pointerDrag.current = {
          payload,
          label,
          clientX: e.clientX,
          clientY: e.clientY,
          altKey: e.altKey,
          featureTarget,
        };
      }
    },
    { signal: listeners.signal },
  );
  window.addEventListener(
    "pointerup",
    (e) => {
      if (e.pointerId !== id) return;
      const drop = moved ? { payload, clientX: e.clientX, clientY: e.clientY, altKey: e.altKey } : null;
      finish();
      if (drop)
        document
          .elementFromPoint(e.clientX, e.clientY)
          ?.dispatchEvent(
            new CustomEvent<PointerDrop>("vile-pointer-drop", { bubbles: true, detail: drop }),
          );
    },
    { signal: listeners.signal },
  );
  window.addEventListener(
    "pointercancel",
    (e) => {
      if (e.pointerId === id) finish();
    },
    { signal: listeners.signal },
  );
  source.addEventListener("lostpointercapture", finish, { signal: listeners.signal });
  window.addEventListener(
    "keydown",
    (e) => {
      if (e.key === "Escape") finish();
      else if (e.key === "Alt" && pointerDrag.current)
        pointerDrag.current = { ...pointerDrag.current, altKey: true };
    },
    { signal: listeners.signal },
  );
  window.addEventListener(
    "keyup",
    (e) => {
      if (e.key === "Alt" && pointerDrag.current)
        pointerDrag.current = { ...pointerDrag.current, altKey: false };
    },
    { signal: listeners.signal },
  );
  window.addEventListener("blur", finish, { signal: listeners.signal });
}

export function pointerDrop(node: Element, accept: (drop: PointerDrop) => boolean) {
  const receive = (event: Event) => {
    if (accept((event as CustomEvent<PointerDrop>).detail)) event.stopPropagation();
  };
  node.addEventListener("vile-pointer-drop", receive);
  return { destroy: () => node.removeEventListener("vile-pointer-drop", receive) };
}

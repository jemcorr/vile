const scopes: HTMLElement[] = [];

/** Keep keyboard focus inside a transient dialog or menu and return it on close. */
export function focusScope(node: HTMLElement, onEscape: () => void) {
  scopes.push(node);
  const previous = document.activeElement instanceof HTMLElement ? document.activeElement : null;
  const controls = () =>
    [
      ...node.querySelectorAll<HTMLElement>(
        'button:not(:disabled), input:not(:disabled), select:not(:disabled), textarea:not(:disabled), [tabindex]:not([tabindex="-1"])',
      ),
    ].filter((item) => !item.closest("[hidden], .hidden, details:not([open])"));
  queueMicrotask(() => {
    if (scopes.at(-1) === node) (controls()[0] ?? node).focus();
  });
  function keydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      event.stopPropagation();
      onEscape();
    } else if (event.key === "Tab") {
      const items = controls();
      if (!items.length) {
        event.preventDefault();
        return;
      }
      const first = items[0]!;
      const last = items[items.length - 1]!;
      if (event.shiftKey && document.activeElement === first) {
        event.preventDefault();
        last.focus();
      } else if (!event.shiftKey && document.activeElement === last) {
        event.preventDefault();
        first.focus();
      }
    }
  }
  function keepFocus(event: FocusEvent) {
    if (scopes.at(-1) === node && event.target instanceof Node && !node.contains(event.target))
      (controls()[0] ?? node).focus();
  }
  node.addEventListener("keydown", keydown);
  document.addEventListener("focusin", keepFocus);
  return {
    destroy() {
      scopes.splice(scopes.indexOf(node), 1);
      node.removeEventListener("keydown", keydown);
      document.removeEventListener("focusin", keepFocus);
      if (previous?.isConnected) queueMicrotask(() => previous.focus());
    },
  };
}

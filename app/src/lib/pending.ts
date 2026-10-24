// Pending-state helpers for awaited Rust calls.
//
// Keep the previous content on screen while a call is in flight. Show a
// pending indicator only when the call is still running after PENDING_DELAY_MS,
// so fast calls never flash a loading state.

export const PENDING_DELAY_MS = 200;

export interface PendingTracker {
  /** Starts a request and returns its id. Any earlier request becomes stale. */
  start(): number;
  /** Settles request `id`. Returns false when a newer request has started since. */
  finish(id: number): boolean;
  /** Invalidates every in-flight request and hides the indicator. */
  cancel(): void;
}

export function createPendingTracker(
  onPending: (pending: boolean) => void,
  delayMs: number = PENDING_DELAY_MS,
): PendingTracker {
  let seq = 0;
  let timer: ReturnType<typeof setTimeout> | null = null;

  const clear = (): void => {
    if (timer !== null) {
      clearTimeout(timer);
      timer = null;
    }
  };

  return {
    start() {
      const id = ++seq;
      clear();
      timer = setTimeout(() => {
        timer = null;
        onPending(true);
      }, delayMs);
      return id;
    },
    finish(id) {
      if (id !== seq) return false;
      clear();
      onPending(false);
      return true;
    },
    cancel() {
      seq++;
      clear();
      onPending(false);
    },
  };
}

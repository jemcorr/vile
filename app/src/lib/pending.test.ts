import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { createPendingTracker, PENDING_DELAY_MS } from "./pending";

describe("createPendingTracker", () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => vi.useRealTimers());

  it("does not show pending for a call that settles within the delay", () => {
    const seen: boolean[] = [];
    const tracker = createPendingTracker((p) => seen.push(p));
    const id = tracker.start();
    vi.advanceTimersByTime(PENDING_DELAY_MS - 1);
    expect(tracker.finish(id)).toBe(true);
    vi.advanceTimersByTime(1000);
    expect(seen).not.toContain(true);
  });

  it("shows pending once the call outlasts the delay", () => {
    const seen: boolean[] = [];
    const tracker = createPendingTracker((p) => seen.push(p));
    const id = tracker.start();
    vi.advanceTimersByTime(PENDING_DELAY_MS);
    expect(seen).toEqual([true]);
    tracker.finish(id);
    expect(seen).toEqual([true, false]);
  });

  it("ignores a stale reply and keeps the newer request's timer", () => {
    const seen: boolean[] = [];
    const tracker = createPendingTracker((p) => seen.push(p));
    const first = tracker.start();
    vi.advanceTimersByTime(150);
    const second = tracker.start();
    expect(tracker.finish(first)).toBe(false);
    expect(seen).toEqual([]);
    vi.advanceTimersByTime(PENDING_DELAY_MS - 1);
    expect(seen).toEqual([]);
    expect(tracker.finish(second)).toBe(true);
    expect(seen).toEqual([false]);
  });

  it("cancel invalidates in-flight requests and hides pending", () => {
    const seen: boolean[] = [];
    const tracker = createPendingTracker((p) => seen.push(p));
    const id = tracker.start();
    vi.advanceTimersByTime(PENDING_DELAY_MS);
    tracker.cancel();
    expect(seen).toEqual([true, false]);
    expect(tracker.finish(id)).toBe(false);
  });
});

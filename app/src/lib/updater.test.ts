import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { UpdateHandle, UpdaterPorts } from "./updater.svelte";

type Updates = typeof import("./updater.svelte");

const DAY_MS = 24 * 60 * 60 * 1000;

function stubStorage(initial: Record<string, string> = {}): Map<string, string> {
  const values = new Map(Object.entries(initial));
  vi.stubGlobal("localStorage", {
    getItem: (key: string) => values.get(key) ?? null,
    setItem: (key: string, value: string) => {
      values.set(key, value);
    },
    removeItem: (key: string) => {
      values.delete(key);
    },
  });
  return values;
}

function fakeUpdate(
  version: string,
  overrides: { download?: () => Promise<void>; install?: () => Promise<void> } = {},
) {
  return {
    version,
    download: vi.fn(overrides.download ?? (() => Promise.resolve())),
    install: vi.fn(overrides.install ?? (() => Promise.resolve())),
    close: vi.fn(() => Promise.resolve()),
  };
}

function fakePorts() {
  return {
    check: vi.fn((): Promise<UpdateHandle | null> => Promise.resolve(null)),
    currentVersion: vi.fn(() => Promise.resolve("0.4.0")),
    updateSupport: vi.fn(() => Promise.resolve()),
    restartApp: vi.fn(() => Promise.resolve()),
    notify: vi.fn(),
    hasUnsavedWork: vi.fn(() => false),
    openRestartDialog: vi.fn(),
    warn: vi.fn(),
  };
}

let updates: Updates;
let ports: ReturnType<typeof fakePorts>;

async function load(): Promise<Updates> {
  vi.resetModules();
  return import("./updater.svelte");
}

beforeEach(async () => {
  stubStorage();
  vi.useFakeTimers();
  updates = await load();
  ports = fakePorts();
  updates.configureUpdater(ports as Partial<UpdaterPorts>);
});

afterEach(() => {
  vi.useRealTimers();
  vi.unstubAllGlobals();
});

describe("automatic checks", () => {
  it("stay off when the key is missing, even if the timers run", async () => {
    expect(updates.updater.autoChecks).toBe(false);
    const stop = updates.startAutoChecks();
    await vi.advanceTimersByTimeAsync(2 * DAY_MS);
    stop();
    expect(ports.check).not.toHaveBeenCalled();
  });

  it("are on only for the exact value on, and unreadable storage means off", async () => {
    expect(updates.parseAutoChecks("on")).toBe(true);
    for (const value of ["off", "ON", "true", "", null, undefined]) {
      expect(updates.parseAutoChecks(value)).toBe(false);
    }
    stubStorage({ "vile.updateChecks": "on" });
    expect((await load()).updater.autoChecks).toBe(true);
    vi.unstubAllGlobals();
    vi.stubGlobal("localStorage", {
      getItem: () => {
        throw new Error("blocked");
      },
    });
    expect((await load()).updater.autoChecks).toBe(false);
  });

  it("check 10 seconds after start, then every 24 hours, when on", async () => {
    updates.setAutoChecks(true);
    const stop = updates.startAutoChecks();
    await vi.advanceTimersByTimeAsync(9_999);
    expect(ports.check).not.toHaveBeenCalled();
    await vi.advanceTimersByTimeAsync(1);
    expect(ports.check).toHaveBeenCalledTimes(1);
    await vi.advanceTimersByTimeAsync(DAY_MS);
    expect(ports.check).toHaveBeenCalledTimes(2);
    stop();
  });

  it("stay silent when they fail", async () => {
    updates.setAutoChecks(true);
    ports.check.mockRejectedValueOnce(new Error("offline"));
    await updates.checkForUpdates({ manual: false });
    expect(ports.notify).not.toHaveBeenCalled();
    expect(ports.warn).toHaveBeenCalledOnce();
  });

  it("announce each downloaded version once", async () => {
    const first = fakeUpdate("0.4.1");
    const second = fakeUpdate("0.4.1");
    ports.check.mockResolvedValueOnce(first).mockResolvedValueOnce(second);
    updates.setAutoChecks(true);
    await updates.checkForUpdates({ manual: false });
    await updates.checkForUpdates({ manual: false });
    const ready = ports.notify.mock.calls.filter(([text]) => String(text).includes("is ready"));
    expect(ready).toHaveLength(1);
    expect(ready[0]?.[1]).toBe(false);
    expect(ready[0]?.[4]).toBe(true);
    expect(first.download).toHaveBeenCalledOnce();
    expect(second.download).not.toHaveBeenCalled();
    expect(second.close).toHaveBeenCalledOnce();
    expect(updates.updater.readyVersion).toBe("0.4.1");
  });
});

describe("manual checks", () => {
  it("say the running version is up to date", async () => {
    await updates.checkForUpdates({ manual: true });
    expect(ports.notify).toHaveBeenLastCalledWith("vile 0.4.0 is up to date");
  });

  it("show an error toast with the detail when the check fails", async () => {
    ports.check.mockRejectedValueOnce("network down");
    await updates.checkForUpdates({ manual: true });
    expect(ports.notify).toHaveBeenLastCalledWith(
      "Could not check for updates",
      true,
      null,
      "network down",
    );
  });

  it("show the update_support reason and never check in a development build", async () => {
    ports.updateSupport.mockRejectedValueOnce("Development builds do not update");
    await updates.checkForUpdates({ manual: true });
    expect(ports.notify).toHaveBeenCalledWith("Development builds do not update");
    expect(ports.check).not.toHaveBeenCalled();
  });

  it("work while automatic checks are off", async () => {
    await updates.checkForUpdates({ manual: true });
    expect(ports.check).toHaveBeenCalledOnce();
  });
});

describe("restart", () => {
  it("opens the unsaved dialog and does not install while work is unsaved", async () => {
    const update = fakeUpdate("0.4.1");
    ports.check.mockResolvedValueOnce(update);
    ports.hasUnsavedWork.mockReturnValue(true);
    await updates.checkForUpdates({ manual: true });
    await updates.requestRestart();
    expect(ports.openRestartDialog).toHaveBeenCalledOnce();
    expect(update.install).not.toHaveBeenCalled();
    expect(ports.restartApp).not.toHaveBeenCalled();
  });

  it("installs, then restarts with the discard choice", async () => {
    const update = fakeUpdate("0.4.1");
    ports.check.mockResolvedValueOnce(update);
    await updates.checkForUpdates({ manual: true });
    await updates.restartToUpdate(true);
    expect(update.install).toHaveBeenCalledOnce();
    expect(ports.restartApp).toHaveBeenCalledWith(true);
  });

  it("drops the cached update after an install failure, so the next check downloads it again", async () => {
    const broken = fakeUpdate("0.4.1", {
      install: () => Promise.reject(new Error("bad signature")),
    });
    const retry = fakeUpdate("0.4.1");
    ports.check.mockResolvedValueOnce(broken).mockResolvedValueOnce(retry);
    await updates.checkForUpdates({ manual: true });
    await updates.restartToUpdate(false);
    expect(ports.notify).toHaveBeenLastCalledWith(
      "Update not installed",
      true,
      null,
      "bad signature",
    );
    expect(broken.close).toHaveBeenCalledOnce();
    expect(ports.restartApp).not.toHaveBeenCalled();
    expect(updates.updater.readyVersion).toBeNull();
    await updates.checkForUpdates({ manual: true });
    expect(retry.download).toHaveBeenCalledOnce();
  });

  it("explains the palette restart and toggle commands", async () => {
    expect(updates.updateReason("restartToUpdate")).toBe("No update is ready");
    expect(updates.updateReason("autoUpdatesOff")).toBe("Automatic checks are already off");
    expect(updates.updateReason("autoUpdatesOn")).toBe("");
    updates.setAutoChecks(true);
    expect(updates.updateReason("autoUpdatesOn")).toBe("Automatic checks are already on");
    expect(updates.updateReason("checkUpdates")).toBe("");
  });
});

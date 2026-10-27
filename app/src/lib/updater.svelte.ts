// Update checks and the restart into a downloaded update. The Rust side verifies each
// package's signature and signed version. This module decides when to check, what the
// user sees, and when a restart may run. Automatic checks are opt-in: they run only
// while localStorage "vile.updateChecks" is "on".
import { getVersion } from "@tauri-apps/api/app";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { api, errorText } from "./api";
import { notify, type ToastAction } from "./state.svelte";

export const UPDATE_CHECKS_KEY = "vile.updateChecks";
const FIRST_CHECK_MS = 10_000;
const CHECK_INTERVAL_MS = 24 * 60 * 60 * 1000;
const CHECK_TIMEOUT_MS = 30_000;

export type UpdatePhase = "idle" | "checking" | "downloading" | "error";

/** The parts of a plugin Update this module uses. Tests pass fakes of this shape. */
export type UpdateHandle = Pick<Update, "version" | "download" | "install" | "close">;

/** Everything this module reaches outside itself. Tests replace these with fakes. */
export interface UpdaterPorts {
  check(options: { timeout: number }): Promise<UpdateHandle | null>;
  currentVersion(): Promise<string>;
  updateSupport(): Promise<void>;
  restartApp(discard: boolean): Promise<void>;
  notify(
    text: string,
    error?: boolean,
    action?: ToastAction | null,
    detail?: string,
    sticky?: boolean,
  ): void;
  hasUnsavedWork(): boolean;
  openRestartDialog(): void;
  warn(message: string, error: unknown): void;
}

/** Only the exact value "on" enables automatic checks. Anything else means off. */
export function parseAutoChecks(value: string | null | undefined): boolean {
  return value === "on";
}

function storage(): Storage | null {
  try {
    return typeof localStorage === "undefined" ? null : localStorage;
  } catch {
    return null;
  }
}

/** Blocked or throwing storage means automatic checks are off. */
function readAutoChecks(): boolean {
  try {
    return parseAutoChecks(storage()?.getItem(UPDATE_CHECKS_KEY) ?? null);
  } catch {
    return false;
  }
}

export const updater = $state({
  phase: "idle" as UpdatePhase,
  /** The version downloaded and waiting for a restart, or null. */
  readyVersion: null as string | null,
  autoChecks: readAutoChecks(),
});

const livePorts: UpdaterPorts = {
  check: (options) => check(options),
  currentVersion: () => getVersion(),
  updateSupport: () => api.updateSupport(),
  restartApp: (discard) => api.restartApp(discard),
  notify: (text, error, action, detail, sticky) => notify(text, error, action, detail, sticky),
  hasUnsavedWork: () => false,
  openRestartDialog: () => {},
  warn: (message, error) => console.warn(message, error),
};

let ports: UpdaterPorts = livePorts;

/** Replaces some ports. The app supplies the unsaved-work hooks at startup. */
export function configureUpdater(next: Partial<UpdaterPorts>): void {
  ports = { ...ports, ...next };
}

/** The downloaded update handle. The plugin keeps its bytes until install. */
let pending: UpdateHandle | null = null;
/** True once install succeeded and only the restart is left. */
let installed = false;
/** The version the automatic ready toast last named. */
let announced: string | null = null;
let restarting = false;

export function autoChecksEnabled(): boolean {
  return updater.autoChecks;
}

/** Turns automatic checks on or off and remembers the choice. Manual checks always work. */
export function setAutoChecks(on: boolean): void {
  try {
    storage()?.setItem(UPDATE_CHECKS_KEY, on ? "on" : "off");
  } catch {
    /* Storage that throws is ignored; the choice then lasts for this session only. */
  }
  updater.autoChecks = on;
}

/** Why a palette update command cannot run now, or "" when it can. */
export function updateReason(id: string): string {
  if (id === "restartToUpdate") return updater.readyVersion ? "" : "No update is ready";
  if (id === "autoUpdatesOn") return updater.autoChecks ? "Automatic checks are already on" : "";
  if (id === "autoUpdatesOff") return updater.autoChecks ? "" : "Automatic checks are already off";
  return "";
}

function fail(manual: boolean, text: string, error: unknown): void {
  updater.phase = "error";
  if (manual) ports.notify(text, true, null, errorText(error));
  else ports.warn(text, error);
}

/** Closes the cached update and forgets it, so the next check downloads it again. */
async function dropPending(): Promise<void> {
  const old = pending;
  pending = null;
  installed = false;
  updater.readyVersion = null;
  await old?.close().catch(() => undefined);
}

function announceReady(version: string, manual: boolean): void {
  if (!manual && announced === version) return;
  announced = version;
  ports.notify(
    `vile ${version} is ready`,
    false,
    { label: "Restart to update", run: () => void requestRestart() },
    "",
    true,
  );
}

/**
 * Checks for a newer release and downloads it in the background. Automatic checks
 * do nothing while they are off, and they stay silent when they fail.
 */
export async function checkForUpdates(options: { manual: boolean }): Promise<void> {
  const { manual } = options;
  if (!manual && !updater.autoChecks) return;
  try {
    await ports.updateSupport();
  } catch (error) {
    if (manual) ports.notify(errorText(error));
    return;
  }
  if (updater.phase === "checking" || updater.phase === "downloading") {
    if (manual) ports.notify("Already checking for updates");
    return;
  }
  updater.phase = "checking";
  if (manual) ports.notify("Checking for updates…");
  let update: UpdateHandle | null;
  try {
    update = await ports.check({ timeout: CHECK_TIMEOUT_MS });
  } catch (error) {
    fail(manual, "Could not check for updates", error);
    return;
  }
  if (!update) {
    updater.phase = "idle";
    if (manual) {
      const current = await ports.currentVersion().catch(() => null);
      ports.notify(current ? `vile ${current} is up to date` : "vile is up to date");
    }
    return;
  }
  if (update.version === updater.readyVersion) {
    await update.close().catch(() => undefined);
    updater.phase = "idle";
    if (manual) announceReady(update.version, true);
    return;
  }
  await dropPending();
  pending = update;
  updater.phase = "downloading";
  try {
    await update.download();
  } catch (error) {
    await dropPending();
    fail(manual, "Could not download the update", error);
    return;
  }
  updater.phase = "idle";
  updater.readyVersion = update.version;
  announceReady(update.version, manual);
}

/** Restarts into the downloaded update. Unsaved work opens the dialog first. */
export async function requestRestart(): Promise<void> {
  if (!updater.readyVersion) {
    ports.notify("No update is ready");
    return;
  }
  if (ports.hasUnsavedWork()) {
    ports.openRestartDialog();
    return;
  }
  await restartToUpdate(false);
}

/**
 * Installs the downloaded update, then restarts the app. Discarding drops the unsaved
 * changes in the restart. A failed install keeps the current version running.
 */
export async function restartToUpdate(discard: boolean): Promise<void> {
  const update = pending;
  if (!update || !updater.readyVersion || restarting) return;
  restarting = true;
  try {
    if (!installed) {
      try {
        await update.install();
        installed = true;
      } catch (error) {
        await dropPending();
        updater.phase = "error";
        ports.notify("Update not installed", true, null, errorText(error));
        return;
      }
    }
    try {
      await ports.restartApp(discard);
    } catch (error) {
      ports.notify(errorText(error), true);
    }
  } finally {
    restarting = false;
  }
}

/**
 * Runs automatic checks: the first one 10 seconds after the call, then one every 24
 * hours. Returns the function that stops them.
 */
export function startAutoChecks(): () => void {
  let stopped = false;
  let interval: ReturnType<typeof setInterval> | undefined;
  const run = (): void => void checkForUpdates({ manual: false });
  const first = setTimeout(() => {
    if (stopped) return;
    run();
    interval = setInterval(run, CHECK_INTERVAL_MS);
  }, FIRST_CHECK_MS);
  return () => {
    stopped = true;
    clearTimeout(first);
    if (interval !== undefined) clearInterval(interval);
  };
}

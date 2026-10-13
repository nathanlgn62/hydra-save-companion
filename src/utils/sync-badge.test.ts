import { describe, expect, it } from "vitest";
import { getGameStatusCategory, getSyncBadgeConfig } from "./sync-badge";

describe("sync-badge utils", () => {
  it("should return Not Found if ludasaviPathExists is false", () => {
    const config = getSyncBadgeConfig(false, null, null, true);
    expect(config.label).toBe("Not Found");
    expect(config.variant).toBe("neutral");

    const category = getGameStatusCategory(false, null, null, true);
    expect(category).toBe("no-save");
  });

  it("should return Not Synced if cloud is disconnected and local save exists", () => {
    const config = getSyncBadgeConfig(true, "13/10/2026 10:00", null, false);
    expect(config.label).toBe("Not Synced");
    expect(config.variant).toBe("neutral");

    const category = getGameStatusCategory(true, "13/10/2026 10:00", null, false);
    expect(category).toBe("never-synced");
  });

  it("should return To Download when local save is missing but cloud save exists", () => {
    const config = getSyncBadgeConfig(true, null, "13/10/2026 10:00", true);
    expect(config.label).toBe("To Download");
    expect(config.variant).toBe("info");

    const category = getGameStatusCategory(true, null, "13/10/2026 10:00", true);
    expect(category).toBe("to-download");
  });

  it("should return Never Synced when local save exists but cloud save is absent", () => {
    const config = getSyncBadgeConfig(true, "13/10/2026 10:00", null, true);
    expect(config.label).toBe("Never Synced");
    expect(config.variant).toBe("warning");

    const category = getGameStatusCategory(true, "13/10/2026 10:00", null, true);
    expect(category).toBe("never-synced");
  });

  it("should return Up to Date when local and cloud saves are at the same time", () => {
    const time = "13/10/2026 10:00";
    const config = getSyncBadgeConfig(true, time, time, true);
    expect(config.label).toBe("Up to Date");
    expect(config.variant).toBe("success");

    const category = getGameStatusCategory(true, time, time, true);
    expect(category).toBe("up-to-date");
  });

  it("should return Local Newer when local save is newer", () => {
    const local = "13/10/2026 12:00";
    const remote = "13/10/2026 10:00";
    const config = getSyncBadgeConfig(true, local, remote, true);
    expect(config.label).toBe("Local Newer");
    expect(config.variant).toBe("info");

    const category = getGameStatusCategory(true, local, remote, true);
    expect(category).toBe("local-newer");
  });

  it("should return Cloud Newer when remote save is newer", () => {
    const local = "13/10/2026 08:00";
    const remote = "13/10/2026 12:00";
    const config = getSyncBadgeConfig(true, local, remote, true);
    expect(config.label).toBe("Cloud Newer");
    expect(config.variant).toBe("warning");

    const category = getGameStatusCategory(true, local, remote, true);
    expect(category).toBe("cloud-newer");
  });
});

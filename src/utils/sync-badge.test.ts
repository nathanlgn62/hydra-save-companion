import { describe, expect, it } from "vitest";
import { getGameStatusCategory, getSyncBadgeConfig } from "./sync-badge";

describe("sync-badge utils", () => {
  it("should return Introuvable if ludasaviPathExists is false", () => {
    const config = getSyncBadgeConfig(false, null, null, true);
    expect(config.label).toBe("Introuvable");
    expect(config.variant).toBe("neutral");

    const category = getGameStatusCategory(false, null, null, true);
    expect(category).toBe("no-save");
  });

  it("should return Non synchronisé if cloud is disconnected and local save exists", () => {
    const config = getSyncBadgeConfig(true, "13/10/2026 10:00", null, false);
    expect(config.label).toBe("Non synchronisé");
    expect(config.variant).toBe("neutral");

    const category = getGameStatusCategory(true, "13/10/2026 10:00", null, false);
    expect(category).toBe("never-synced");
  });

  it("should return À télécharger when local save is missing but cloud save exists", () => {
    const config = getSyncBadgeConfig(true, null, "13/10/2026 10:00", true);
    expect(config.label).toBe("À télécharger");
    expect(config.variant).toBe("info");

    const category = getGameStatusCategory(true, null, "13/10/2026 10:00", true);
    expect(category).toBe("to-download");
  });

  it("should return Jamais sync when local save exists but cloud save is absent", () => {
    const config = getSyncBadgeConfig(true, "13/10/2026 10:00", null, true);
    expect(config.label).toBe("Jamais sync");
    expect(config.variant).toBe("warning");

    const category = getGameStatusCategory(true, "13/10/2026 10:00", null, true);
    expect(category).toBe("never-synced");
  });

  it("should return À jour when local and cloud saves are at the same time", () => {
    const time = "13/10/2026 10:00";
    const config = getSyncBadgeConfig(true, time, time, true);
    expect(config.label).toBe("À jour");
    expect(config.variant).toBe("success");

    const category = getGameStatusCategory(true, time, time, true);
    expect(category).toBe("up-to-date");
  });

  it("should return Local plus récent when local save is newer", () => {
    const local = "13/10/2026 12:00";
    const remote = "13/10/2026 10:00";
    const config = getSyncBadgeConfig(true, local, remote, true);
    expect(config.label).toBe("Local plus récent");
    expect(config.variant).toBe("info");

    const category = getGameStatusCategory(true, local, remote, true);
    expect(category).toBe("local-newer");
  });

  it("should return Cloud plus récent when remote save is newer", () => {
    const local = "13/10/2026 08:00";
    const remote = "13/10/2026 12:00";
    const config = getSyncBadgeConfig(true, local, remote, true);
    expect(config.label).toBe("Cloud plus récent");
    expect(config.variant).toBe("warning");

    const category = getGameStatusCategory(true, local, remote, true);
    expect(category).toBe("cloud-newer");
  });
});

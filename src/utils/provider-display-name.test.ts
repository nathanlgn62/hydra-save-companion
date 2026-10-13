import { describe, expect, it } from "vitest";
import { getProviderDisplayName } from "./provider-display-name";

describe("provider-display-name utils", () => {
  it("should return Google Drive for google-drive", () => {
    expect(getProviderDisplayName("google-drive")).toBe("Google Drive");
  });

  it("should return Dropbox for dropbox", () => {
    expect(getProviderDisplayName("dropbox")).toBe("Dropbox");
  });

  it("should return Proton Drive for proton-drive", () => {
    expect(getProviderDisplayName("proton-drive")).toBe("Proton Drive");
  });

  it("should return Mega for mega", () => {
    expect(getProviderDisplayName("mega")).toBe("Mega");
  });

  it("should return fallback for unknown or null", () => {
    expect(getProviderDisplayName(null)).toBe("Cloud Connected");
    expect(getProviderDisplayName("unknown-provider")).toBe("Cloud Connected");
  });
});

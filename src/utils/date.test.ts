import { describe, expect, it } from "vitest";
import { formatDate, parseSaveDate } from "./date";

describe("date utils", () => {
  describe("parseSaveDate", () => {
    it("should return null for empty, null, undefined or special values", () => {
      expect(parseSaveDate(null)).toBeNull();
      expect(parseSaveDate(undefined)).toBeNull();
      expect(parseSaveDate("")).toBeNull();
      expect(parseSaveDate("Jamais")).toBeNull();
      expect(parseSaveDate("Aucune")).toBeNull();
    });

    it("should parse standard French formatted date (DD/MM/YYYY HH:mm)", () => {
      const parsed = parseSaveDate("13/10/2026 14:30");
      expect(parsed).not.toBeNull();
      const date = new Date(parsed!);
      expect(date.getFullYear()).toBe(2026);
      expect(date.getMonth()).toBe(9); // 0-indexed: October is 9
      expect(date.getDate()).toBe(13);
      expect(date.getHours()).toBe(14);
      expect(date.getMinutes()).toBe(30);
    });

    it("should parse ISO date string as fallback", () => {
      const iso = "2026-10-13T12:00:00.000Z";
      const parsed = parseSaveDate(iso);
      expect(parsed).toBe(new Date(iso).getTime());
    });
  });

  describe("formatDate", () => {
    it("should return 'Never' for empty or null dates", () => {
      expect(formatDate(null)).toBe("Never");
      expect(formatDate(undefined)).toBe("Never");
      expect(formatDate("")).toBe("Never");
    });

    it("should format ISO date strings properly", () => {
      const formatted = formatDate("2026-10-13T14:30:00Z");
      expect(formatted).toMatch(/\d{2}\/\d{2}/);
    });

    it("should return raw string if unparseable", () => {
      expect(formatDate("non-date")).toBe("non-date");
    });
  });
});

export const parseSaveDate = (dateStr?: string | null): number | null => {
  if (!dateStr || dateStr === "Jamais" || dateStr === "Aucune") return null;

  // Format Rust "DD/MM/YYYY HH:mm"
  const customFormatRegex = /^(\d{2})\/(\d{2})\/(\d{4})\s+(\d{2}):(\d{2})$/;
  const match = dateStr.match(customFormatRegex);

  if (match) {
    const [, day, month, year, hours, minutes] = match;
    return new Date(
      Number(year),
      Number(month) - 1,
      Number(day),
      Number(hours),
      Number(minutes),
    ).getTime();
  }

  // Fallback ISO
  const parsed = new Date(dateStr).getTime();
  return isNaN(parsed) ? null : parsed;
};

export const formatDate = (dateStr?: string | null) => {
  if (!dateStr) return "Jamais";
  try {
    const date = new Date(dateStr);
    if (isNaN(date.getTime())) return dateStr;
    return new Intl.DateTimeFormat("fr-FR", {
      day: "2-digit",
      month: "2-digit",
      hour: "2-digit",
      minute: "2-digit",
    }).format(date);
  } catch {
    return dateStr;
  }
};

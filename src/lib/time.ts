/// Относительное время для свежего, абсолютная дата для старого
/// (рекомендация PatternFly: <1ч — относительное, дальше — точное).
export function ago(unixSeconds: number | null): string {
  if (!unixSeconds) return "";
  const diff = Date.now() / 1000 - unixSeconds;
  if (diff < 60) return "сейчас";
  if (diff < 3600) return `${Math.floor(diff / 60)} мин`;
  if (diff < 86400) return `${Math.floor(diff / 3600)} ч`;

  const d = new Date(unixSeconds * 1000);
  const sameYear = new Date().getFullYear() === d.getFullYear();
  return d.toLocaleDateString(
    "ru-RU",
    sameYear
      ? { day: "numeric", month: "short" }
      : { day: "numeric", month: "short", year: "2-digit" },
  );
}

/// Бейдж-счётчик не длиннее двух цифр (Setproduct: never past two digits).
export function cap(n: number): string {
  return n > 99 ? "99+" : String(n);
}

/// Три уровня важности из ntfy priority (1..5) → цветовая полоса.
export function tier(priority: number): "high" | "low" | "" {
  if (priority >= 4) return "high";
  if (priority <= 2) return "low";
  return "";
}

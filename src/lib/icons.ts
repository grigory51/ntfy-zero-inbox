/// Подбор иконки по смыслу имени топика. Эмодзи здесь сознательно:
/// нативный рендер, «скевоморфный» вид, ноль ассетов.
const RULES: Array<[RegExp, string]> = [
  [/backup|dump|archive/i, "💾"],
  [/ci|build|pipeline|deploy|release/i, "🚀"],
  [/alert|alarm|critical|incident/i, "🚨"],
  [/error|fail|crash/i, "💥"],
  [/warn/i, "⚠️"],
  [/server|host|infra|vps/i, "🖥️"],
  [/db|sql|postgres|mysql|mongo/i, "🗄️"],
  [/net|vpn|wifi|dns/i, "🌐"],
  [/security|auth|ssl|cert/i, "🛡️"],
  [/home|iot|hass|smart/i, "🏠"],
  [/money|pay|billing|invoice|bank/i, "💰"],
  [/mail|smtp|email/i, "✉️"],
  [/log|journal|audit/i, "📜"],
  [/test|qa/i, "🧪"],
  [/download|torrent|fetch/i, "📥"],
  [/update|upgrade|patch/i, "🔄"],
  [/cron|schedule|timer|job/i, "⏰"],
  [/video|stream|cam/i, "🎬"],
  [/music|audio/i, "🎵"],
  [/weather/i, "⛅"],
  [/health|fit/i, "❤️"],
  [/git|repo|commit/i, "🌿"],
  [/docker|container|k8s|kube/i, "📦"],
  [/print/i, "🖨️"],
  [/disk|storage|space/i, "💿"],
  [/power|battery|ups/i, "🔋"],
  [/bot|agent|ai|llm/i, "🤖"],
];

export function iconFor(topic: string): string {
  for (const [re, icon] of RULES) {
    if (re.test(topic)) return icon;
  }
  return "🔔";
}

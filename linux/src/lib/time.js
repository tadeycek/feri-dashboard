const pad = (n) => String(n).padStart(2, "0");

export const dayKey = (d) => `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
export const hm = (d) => `${pad(d.getHours())}:${pad(d.getMinutes())}`;
export const minutesOfDay = (d) => d.getHours() * 60 + d.getMinutes();

export const addDays = (d, n) => {
  const x = new Date(d);
  x.setDate(x.getDate() + n);
  return x;
};
export const startOfDay = (d) => {
  const x = new Date(d);
  x.setHours(0, 0, 0, 0);
  return x;
};
export const startOfWeek = (d) => addDays(startOfDay(d), -((d.getDay() + 6) % 7));

const L = "en-GB";
const fLong = new Intl.DateTimeFormat(L, { weekday: "long", day: "numeric", month: "long" });
const fWeekday = new Intl.DateTimeFormat(L, { weekday: "short" });
const fDayMonth = new Intl.DateTimeFormat(L, { day: "numeric", month: "long" });
const fDayMonthShort = new Intl.DateTimeFormat(L, { day: "numeric", month: "short" });
const fMonthYear = new Intl.DateTimeFormat(L, { month: "long", year: "numeric" });
const fDayMonthYear = new Intl.DateTimeFormat(L, { day: "numeric", month: "long", year: "numeric" });

/** "Tuesday 6 October" */
export const longDate = (d) => fLong.format(d).replace(",", "");
export const weekdayShort = (d) => fWeekday.format(d).replace(".", "");
export const dayMonth = (d) => fDayMonth.format(d);
export const dayMonthShort = (d) => fDayMonthShort.format(d);

/** "12–16 October 2026" */
export function weekRange(monday, lastDay) {
  const a = monday, b = lastDay;
  if (a.getMonth() === b.getMonth()) return `${a.getDate()}–${b.getDate()} ${fMonthYear.format(b)}`;
  return `${fDayMonth.format(a)} – ${fDayMonthYear.format(b)}`;
}

/** 47 → "47 min", 135 → "2 h 15 min", 3000 → "2 days" */
export function duration(min) {
  min = Math.max(0, Math.round(min));
  if (min < 60) return `${min} min`;
  const h = Math.floor(min / 60);
  if (h >= 24) {
    const d = Math.floor(h / 24);
    return d === 1 ? "1 day" : `${d} days`;
  }
  const m = min % 60;
  return m ? `${h} h ${m} min` : `${h} h`;
}

/** Whole days from today to a YYYY-MM-DD date (negative = past). */
export function daysUntil(dateStr, now) {
  const [y, m, d] = dateStr.split("-").map(Number);
  return Math.round((new Date(y, m - 1, d) - startOfDay(now)) / 86400000);
}

export function dueLabel(n) {
  if (n < 0) return n === -1 ? "1 day overdue" : `${-n} days overdue`;
  if (n === 0) return "today";
  if (n === 1) return "tomorrow";
  return `in ${n} days`;
}

/** plural(3, "note") → "3 notes"; plural(1, "note") → "1 note" */
export const count = (n, one, many = `${one}s`) => `${n} ${n === 1 ? one : many}`;

export function ago(from, now) {
  const min = Math.round((now - from) / 60000);
  if (min < 1) return "just now";
  if (min < 60) return `${min} min ago`;
  const h = Math.round(min / 60);
  if (h < 24) return `${h} h ago`;
  const d = Math.round(h / 24);
  return d === 1 ? "yesterday" : `${d} days ago`;
}

/** "2026-10-06" → "06-10-2026" */
export const numericDate = (iso) => iso.split("-").reverse().join("-");

export function rokDate(r) {
  const [y, m, d] = r.datum.split("-").map(Number);
  return new Date(y, m - 1, d);
}

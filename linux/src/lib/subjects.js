// Display labels are English. The stored ids (kolokvij, oddaja, …) are the vault's own values and stay as they are.
export const KIND = {
  PR: { long: "Lecture", short: "Lec" },
  RV: { long: "Exercises", short: "Ex" },
  SV: { long: "Seminar", short: "Sem" },
  LV: { long: "Lab", short: "Lab" },
  REZ: { long: "Reservation", short: "Res" },
};

export const kindLong = (k) => KIND[k]?.long ?? k;
export const kindShort = (k) => KIND[k]?.short ?? k;
/** Subject colour as a CSS value (CSS variables are defined for light and dark in app.css). */
export const colorOf = (key) => (key ? `var(--c-${key})` : "var(--ink-3)");
/** Which vault folder a timetable kind belongs to. */
export const noteKindFor = (kind) => (kind === "PR" ? "Predavanja" : "Vaje");

export const ROK_TYPES = [
  { id: "kolokvij", label: "Midterm" },
  { id: "izpit", label: "Exam" },
  { id: "oddaja", label: "Assignment" },
  { id: "pomembno", label: "Important" },
];
export const rokTypeLabel = (id) => ROK_TYPES.find((t) => t.id === id)?.label ?? id;

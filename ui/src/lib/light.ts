// The object's light: green verified, amber working, red failed, quartz off. CSS values for --hl.

export type Light = "green" | "amber" | "red" | "off";

export const LIGHT_VAR: Record<Light, string> = {
  green: "var(--color-magsafe-green)",
  amber: "var(--color-magsafe-amber)",
  red: "var(--color-signal-red)",
  off: "var(--color-quartz)",
};

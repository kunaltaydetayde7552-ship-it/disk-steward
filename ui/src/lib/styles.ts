// Shared utility strings for the control family: one source, so every button has the same five states.

export const focusRing = "focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-anodized";
export const btnBase = `inline-flex select-none items-center justify-center gap-2 whitespace-nowrap rounded-key font-medium transition-[background-color,box-shadow,scale,color,opacity] duration-150 ease-platen active:scale-[0.97] disabled:cursor-not-allowed disabled:opacity-45 disabled:active:scale-100 ${focusRing}`;
export const btnPrimary = `${btnBase} h-10 px-5 bg-anodized text-frost shadow-lift hover:bg-anodized/85 active:shadow-press`;
export const btnSecondary = `${btnBase} h-10 px-4 bg-frost text-anodized shadow-lift ring-1 ring-aluminum-deep hover:ring-anodized-muted/50 active:shadow-press`;
export const btnGhost = `${btnBase} h-9 px-3 text-anodized-muted hover:bg-aluminum/80 hover:text-anodized`;
/** a ghost button sized for a single line of micro text (copy path) */
export const btnGhostSmall = `${btnBase} h-8 px-2 text-micro text-anodized-muted hover:bg-aluminum/80 hover:text-anodized`;
export const divided = "divide-y divide-aluminum-deep border-y border-aluminum-deep";
export const screenTitle = "text-display font-light tracking-[-0.02em]";
export const screenLead = "mt-2 max-w-[60ch] text-lead text-anodized-muted";

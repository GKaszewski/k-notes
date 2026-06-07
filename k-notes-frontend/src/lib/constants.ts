const NOTE_COLORS = [
    { name: "DEFAULT",  label: "Default", glass: null,                        borderClass: "",                      swatch: "rgba(255,255,255,0.25)" },
    { name: "RED",      label: "Red",     glass: "rgba(239, 68, 68, 0.22)",   borderClass: "border-red-400/50",     swatch: "rgb(239, 68, 68)" },
    { name: "ORANGE",   label: "Orange",  glass: "rgba(249, 115, 22, 0.22)",  borderClass: "border-orange-400/50",  swatch: "rgb(249, 115, 22)" },
    { name: "YELLOW",   label: "Yellow",  glass: "rgba(234, 179, 8, 0.22)",   borderClass: "border-yellow-400/50",  swatch: "rgb(234, 179, 8)" },
    { name: "GREEN",    label: "Green",   glass: "rgba(34, 197, 94, 0.22)",   borderClass: "border-green-400/50",   swatch: "rgb(34, 197, 94)" },
    { name: "TEAL",     label: "Teal",    glass: "rgba(20, 184, 166, 0.22)",  borderClass: "border-teal-400/50",    swatch: "rgb(20, 184, 166)" },
    { name: "BLUE",     label: "Blue",    glass: "rgba(59, 130, 246, 0.22)",  borderClass: "border-blue-400/50",    swatch: "rgb(59, 130, 246)" },
    { name: "INDIGO",   label: "Indigo",  glass: "rgba(99, 102, 241, 0.22)",  borderClass: "border-indigo-400/50",  swatch: "rgb(99, 102, 241)" },
];

export function getNoteColor(colorName: string | undefined): { glass: string | null; borderClass: string } {
    const color = NOTE_COLORS.find(c => c.name === colorName);
    return color
        ? { glass: color.glass, borderClass: color.borderClass }
        : { glass: null, borderClass: "" };
}

export { NOTE_COLORS };

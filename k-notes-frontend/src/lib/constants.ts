const NOTE_COLORS = [
    { name: "DEFAULT", value: "bg-background border-border", label: "Default" },
    { name: "RED", value: "bg-red-50 border-red-200 dark:bg-red-950/20 dark:border-red-900", label: "Red" },
    { name: "ORANGE", value: "bg-orange-50 border-orange-200 dark:bg-orange-950/20 dark:border-orange-900", label: "Orange" },
    { name: "YELLOW", value: "bg-yellow-50 border-yellow-200 dark:bg-yellow-950/20 dark:border-yellow-900", label: "Yellow" },
    { name: "GREEN", value: "bg-green-50 border-green-200 dark:bg-green-950/20 dark:border-green-900", label: "Green" },
    { name: "TEAL", value: "bg-teal-50 border-teal-200 dark:bg-teal-950/20 dark:border-teal-900", label: "Teal" },
    { name: "BLUE", value: "bg-blue-50 border-blue-200 dark:bg-blue-950/20 dark:border-blue-900", label: "Blue" },
    { name: "INDIGO", value: "bg-indigo-50 border-indigo-200 dark:bg-indigo-950/20 dark:border-indigo-900", label: "Indigo" },
];

export function getNoteColor(colorName: string | undefined): string {
    const color = NOTE_COLORS.find(c => c.name === colorName);
    return color ? color.value : NOTE_COLORS[0].value;
}

export { NOTE_COLORS };

/** Keyboard shortcuts of the full player: one table for the handler's docs and the help page. */
export const SHORTCUTS: { group: string; keys: string; action: string }[] = [
  { group: "Reproducción", keys: "Espacio", action: "Reproducir / pausar" },
  { group: "Reproducción", keys: "N / P", action: "Siguiente / anterior" },
  { group: "Reproducción", keys: "← / →", action: "Retroceder / adelantar 5 s" },
  { group: "Reproducción", keys: "Ctrl + ↑ / ↓", action: "Subir / bajar volumen" },
  { group: "Letra", keys: "L", action: "Mostrar / ocultar la letra" },
  { group: "Letra", keys: "T", action: "Traducir la letra al español" },
  { group: "Ventanas", keys: "M", action: "Mini reproductor" },
  { group: "Ventanas", keys: "E", action: "Ecualizador" },
  { group: "Ventanas", keys: "Ctrl + ,", action: "Ajustes" },
  { group: "Ventanas", keys: "?", action: "Esta ayuda" },
  { group: "Ventanas", keys: "Esc", action: "Cerrar ajustes / ayuda" },
  { group: "Búsqueda y listas", keys: "Ctrl + F", action: "Ir al buscador de la vista" },
  { group: "Búsqueda y listas", keys: "Doble clic", action: "Reproducir desde esa canción" },
  { group: "Búsqueda y listas", keys: "↑ / ↓, rueda", action: "Desplazar listas" },
];

/** True when a key press belongs to a text field and must not trigger shortcuts. */
export const typing = (e: KeyboardEvent) =>
  e.target instanceof HTMLInputElement ||
  e.target instanceof HTMLTextAreaElement ||
  e.target instanceof HTMLSelectElement ||
  (e.target instanceof HTMLElement && e.target.isContentEditable);

export type Entry = { path: string; name: string; directory?: boolean };
export type Settings = { add: string; remove: string; position: number; fromEnd: boolean; numbered: boolean; start: number; step: number; digits: number; hideExtension: boolean };
export type Preview = Entry & { target: string; valid: boolean; error?: string };
export function splitName(name: string, directory = false): [string, string] {
  if (directory) return [name, ''];
  const dot = name.lastIndexOf('.');
  return dot <= 0 ? [name, ''] : [name.slice(0, dot), name.slice(dot)];
}
export function plan(entries: Entry[], s: Settings): Preview[] {
  const planned = entries.map((entry, i) => {
    const [original, ext] = splitName(entry.name, entry.directory);
    let name = original;
    if (s.remove) name = name.split(s.remove).join('');
    const position = Math.max(0, Math.min(name.length, s.fromEnd ? name.length - s.position : s.position));
    const number = s.numbered ? String(s.start + i * s.step).padStart(Math.max(1, Math.min(12, s.digits)), '0') : '';
    name = name.slice(0, position) + s.add + number + name.slice(position);
    const target = name + ext;
    const error = !name || name === '.' || name === '..' || target.includes('/') || target.includes('\\') || target.includes('\0') ? 'Invalid filename' : undefined;
    return { ...entry, target, valid: !error, error };
  });
  const seen = new Map<string, number>();
  for (const row of planned) {
    const parent = row.path.replace(/[\\/][^\\/]+$/, '');
    const key = parent + '/' + row.target.normalize('NFC').toLowerCase();
    seen.set(key, (seen.get(key) ?? 0) + 1);
  }
  return planned.map(row => {
    const parent = row.path.replace(/[\\/][^\\/]+$/, '');
    const key = parent + '/' + row.target.normalize('NFC').toLowerCase();
    const error = row.error ?? (seen.get(key)! > 1 ? 'Duplicate target' : undefined);
    return { ...row, valid: !error, error };
  });
}

export type Entry = { path: string; name: string; directory?: boolean };
export type Settings = { mode: 'add' | 'remove'; kind: 'text' | 'numbering'; text: string; position: number; fromEnd: boolean; start: number; digits: number; hideExtension: boolean };
export type Preview = Entry & { target: string; valid: boolean; error?: string };
export function splitName(name: string, directory = false): [string, string] {
  if (directory) return [name, ''];
  const dot = name.lastIndexOf('.');
  return dot <= 0 ? [name, ''] : [name.slice(0, dot), name.slice(dot)];
}
export function plan(entries: Entry[], s: Settings): Preview[] {
  const planned = entries.map((entry, i) => {
    const [original, ext] = splitName(entry.name, entry.directory);
    const offset = Number.isFinite(s.position) ? Math.max(0, Math.trunc(s.position)) : 0;
    const position = Math.max(0, Math.min(original.length, s.fromEnd ? original.length - offset : offset));
    const digits = Number.isFinite(s.digits) ? Math.max(1, Math.min(12, Math.trunc(s.digits))) : 1;
    const start = Number.isFinite(s.start) ? Math.trunc(s.start) : 1;
    const token = s.kind === 'numbering' ? String(start + i).padStart(digits, '0') : s.text;
    let name = original;
    if (s.mode === 'add') name = original.slice(0, position) + token + original.slice(position);
    else if (s.kind === 'numbering') name = original.slice(0, position) + original.slice(Math.min(original.length, position + digits));
    else if (token) {
      const index = original.indexOf(token, position);
      if (index >= 0) name = original.slice(0, index) + original.slice(index + token.length);
    }
    const target = name + ext;
    const error = !name || name === '.' || name === '..' || /[\\/\0]/.test(target) ? 'Invalid filename' : undefined;
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

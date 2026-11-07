<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';
  import { plan, splitName, type Entry, type Settings } from './renamer';
  let entries: Entry[] = $state([]);
  let status = $state('Ready');
  let settings: Settings = $state({ add: '', remove: '', position: 0, fromEnd: false, numbered: false, start: 1, step: 1, digits: 2, hideExtension: true });
  let preview = $derived(plan(entries, settings));
  let conflicts = $derived(preview.filter(x => !x.valid).length);
  let native = '__TAURI_INTERNALS__' in window;
  async function addPaths() {
    if (!native) { status = 'Browser mode: choose files for preview. Native renaming requires the desktop app.'; return; }
    try {
      const picked = await open({ multiple: true, directory: false });
      if (!picked) return;
      const paths = Array.isArray(picked) ? picked : [picked];
      entries = [...entries, ...paths.map(path => ({ path, name: path.split(/[\\/]/).pop() ?? path }))];
      status = entries.length + ' files selected';
    } catch (e) { status = String(e); }
  }
  async function addFolder() {
    if (!native) { status = 'Folder selection for renaming requires the native app.'; return; }
    try {
      const picked = await open({ directory: true, multiple: false });
      if (!picked || Array.isArray(picked)) return;
      const rows = await invoke<Entry[]>('list_folder', { folder: picked });
      entries = [...entries, ...rows]; status = rows.length + ' items found';
    } catch (e) { status = String(e); }
  }
  async function webFiles(ev: Event) {
    const input = ev.currentTarget as HTMLInputElement;
    const files = Array.from(input.files ?? []);
    entries = [...entries, ...files.map((f, i) => ({ path: 'browser/' + i + '/' + f.name, name: f.name }))];
    status = 'Browser preview only; no files modified'; input.value = '';
  }
  async function apply() {
    if (!native) { status = 'Browser preview only: use the native app to rename files in place.'; return; }
    if (conflicts || !preview.length) return;
    try { const result = await invoke<string>('apply_renames', { changes: preview.filter(p => p.name !== p.target).map(p => ({ from: p.path, to_name: p.target })) }); status = result; entries = []; }
    catch (e) { status = 'Rename failed: ' + String(e); }
  }
  function shown(name: string, dir = false) { return settings.hideExtension ? splitName(name, dir)[0] : name; }
</script>
<svelte:head><title>Enumu — Fast Batch Renamer</title><meta name="description" content="Enumu — lightweight batch filename and folder renaming"/></svelte:head>
<main><header><div><div class="eyebrow">LIGHTWEIGHT FILE UTILITY</div><h1>Enumu<span>.</span></h1><p>Rename anything. Only what you intend.</p></div><div class="chip">{native ? 'Native mode' : 'Web preview mode'}</div></header>
<section class="controls"><h2>Transform</h2><div class="grid"><label>Add Text<input bind:value={settings.add} placeholder="Text to insert" /></label><label>Remove Text<input bind:value={settings.remove} placeholder="Exact text to remove" /></label><label>Character Position<input type="number" min="0" bind:value={settings.position} /></label><label>Count Position From<select bind:value={settings.fromEnd}><option value={false}>Beginning</option><option value={true}>End</option></select></label></div>
<div class="options"><label><input type="checkbox" bind:checked={settings.numbered}/> Add numbering</label><label><input type="checkbox" bind:checked={settings.hideExtension}/> Hide extensions in preview</label></div>
{#if settings.numbered}<div class="grid number"><label>Start<input type="number" bind:value={settings.start}/></label><label>Increment<input type="number" bind:value={settings.step}/></label><label>Digits<input type="number" min="1" max="12" bind:value={settings.digits}/></label></div>{/if}</section>
<section class="files"><div class="files-head"><div><h2>Files <small>{entries.length.toLocaleString()}</small></h2><p>Changes appear instantly before you apply them.</p></div><div class="buttons"><button class="outline" onclick={() => entries = []} disabled={!entries.length}>Clear</button>{#if native}<button class="outline" onclick={addFolder}>Add Folder</button><button onclick={addPaths}>Add Files</button>{:else}<label class="button">Add Files<input type="file" multiple onchange={webFiles} hidden /></label>{/if}</div></div>
<div class="table"><div class="table-header"><span>Original</span><span>Renamed Preview</span></div>{#if !preview.length}<div class="empty"><div class="large">⇧</div>Choose some files to start previewing their new names.</div>{:else}{#each preview.slice(0, 500) as item, i}<div class="item"><span title={item.name}>{shown(item.name, item.directory)}</span><span class:bad={!item.valid} title={item.error ?? item.target}>{shown(item.target, item.directory)}{#if item.error}<em> — {item.error}</em>{/if}</span></div>{/each}{#if preview.length > 500}<div class="more">Showing the first 500 of {preview.length.toLocaleString()} entries</div>{/if}{/if}</div>
<div class="footer"><div class="status">{status}{#if conflicts}<strong> · {conflicts} conflicts</strong>{/if}</div><button class="apply" onclick={apply} disabled={!native || !entries.length || conflicts > 0}>Apply {entries.length.toLocaleString()} Renames</button></div></section>
<footer>Offline first · No accounts · Original extensions preserved</footer></main>
<style>
:global(*){box-sizing:border-box}:global(body){margin:0;background:#0e1015;color:#edf1f7;font-family:Inter,ui-sans-serif,system-ui,-apple-system,'Segoe UI',sans-serif}:global(button),:global(input),:global(select){font:inherit}main{max-width:1060px;padding:56px 24px;margin:auto}header{display:flex;align-items:flex-start;justify-content:space-between;gap:15px;margin-bottom:36px}.eyebrow{letter-spacing:.18em;font-size:10px;color:#8a96ac;font-weight:700}h1{font-size:56px;letter-spacing:-.07em;margin:7px 0 0;line-height:1}h1 span{color:#8a9fff}header p,.files-head p{color:#8791a3;margin:9px 0 0;font-size:14px}.chip{background:#222839;border:1px solid #3a4157;border-radius:999px;padding:8px 13px;font-size:12px;color:#b7c1e1;white-space:nowrap}.controls,.files{background:#171a22;border:1px solid #303541;border-radius:16px;padding:24px;margin-bottom:18px}h2{margin:0 0 21px;font-size:17px;letter-spacing:-.03em}.grid{display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:13px}.grid label{color:#9ba5b7;font-size:12px;font-weight:600}input:not([type=checkbox]),select{display:block;width:100%;background:#11141b;color:#eef1fb;border:1px solid #343947;padding:12px 11px;border-radius:8px;margin-top:8px;outline:none;min-width:0}input:focus,select:focus{border-color:#8a9fff}.options{display:flex;gap:24px;flex-wrap:wrap;margin-top:23px;font-size:13px;color:#cad0db}.options label{display:flex;gap:7px;align-items:center}.options input{accent-color:#879aff}.number{margin-top:18px;grid-template-columns:repeat(3,minmax(0,180px))}.files-head{display:flex;justify-content:space-between;align-items:center;gap:16px;margin-bottom:18px}.files-head h2{margin:0}.files-head small{color:#8c96aa;font-size:12px;margin-left:6px}.buttons{display:flex;gap:8px;align-items:center;flex-wrap:wrap}button,.button{cursor:pointer;border:1px solid #8996f9;border-radius:9px;padding:10px 15px;background:#7685e9;color:#fff;font-size:12px;font-weight:700;display:inline-flex;align-items:center}button.outline{background:#242936;border-color:#3e4657;color:#dae0ef}button:disabled{opacity:.35;cursor:not-allowed}.table{border:1px solid #313644;border-radius:10px;overflow:hidden;min-height:210px}.table-header,.item{display:grid;grid-template-columns:1fr 1fr;gap:14px;padding:12px 15px}.table-header{background:#222633;color:#a8b1c5;font-size:11px;text-transform:uppercase;letter-spacing:.07em;font-weight:700}.item{border-top:1px solid #2b303d;font-size:13px}.item span{white-space:nowrap;overflow:hidden;text-overflow:ellipsis}.item span:nth-child(2){color:#aab7ff}.item .bad,.item em{color:#ff9e9e!important}.empty{text-align:center;color:#8590a4;padding:38px 12px;font-size:13px}.large{font-size:30px;margin-bottom:10px;color:#7d89b2}.more{padding:12px;color:#8d96a8;font-size:12px}.footer{display:flex;justify-content:space-between;align-items:center;gap:20px;padding-top:18px}.status{font-size:12px;color:#94a0b7;overflow-wrap:anywhere}.status strong{color:#ff9e9e}footer{font-size:11px;color:#727e94;text-align:center;padding:12px}@media(max-width:720px){main{padding:25px 12px}.controls,.files{padding:17px}.grid{grid-template-columns:repeat(2,minmax(0,1fr))}.files-head,.footer{flex-direction:column;align-items:stretch}header h1{font-size:44px}.chip{font-size:10px}.buttons{justify-content:flex-start}.table-header,.item{gap:8px;padding:10px;font-size:12px}}
</style>
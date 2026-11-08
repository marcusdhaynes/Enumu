<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';
  import { plan, splitName, type Entry, type Settings } from './renamer';
  let entries: Entry[] = $state([]);
  let status = $state('Ready');
  let settings: Settings = $state({ mode: 'add', kind: 'text', text: '', position: 0, fromEnd: false, start: 1, digits: 2, hideExtension: true });
  let dragOver = $state(false);
  let selected = $state<Set<string>>(new Set());
  let anchor = $state(0);
  let undoDepth = $state(0);
  let busy = $state(false);
  type BatchResult = { count: number; remaining: number; entries: Entry[]; previous: string[] };
  if ('__TAURI_INTERNALS__' in window) invoke<number>('undo_count').then(n => undoDepth = n).catch(() => {});
  let chosen = $derived(entries.filter(e => selected.has(e.path)));
  let selectedPreview = $derived(plan(chosen, settings));
  let targets = $derived(new Map(selectedPreview.map(p => [p.path, p])));
  function append(rows: Entry[]) {
    const seen = new Set(entries.map(e => e.path));
    const fresh = rows.filter(e => !seen.has(e.path));
    entries = [...entries, ...fresh];
    selected = new Set([...selected, ...fresh.map(e => e.path)]);
    status = entries.length + ' files loaded';
  }
  function selectAll() { selected = new Set(entries.map(e => e.path)); }
  function selectNone() { selected = new Set(); }
  function selectRow(path: string, i: number, shift: boolean, additive: boolean) {
    const next = new Set(selected);
    if (shift) {
      if (!additive) next.clear();
      for (let n = Math.min(i, anchor); n <= Math.max(i, anchor); n++) next.add(entries[n].path);
    } else if (additive) {
      next.has(path) ? next.delete(path) : next.add(path);
      anchor = i;
    } else { next.clear(); next.add(path); anchor = i; }
    selected = next;
  }
  function checkRow(path: string, i: number, checked: boolean, shift: boolean) {
    const next = new Set(selected);
    if (shift) for (let n = Math.min(i,anchor); n <= Math.max(i,anchor); n++) checked ? next.add(entries[n].path) : next.delete(entries[n].path);
    else { checked ? next.add(path) : next.delete(path); anchor = i; }
    selected = next;
  }
  function keys(e: KeyboardEvent) {
    if (['INPUT','TEXTAREA','SELECT'].includes((e.target as HTMLElement).tagName)) return;
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'a') { e.preventDefault(); selectAll(); }
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'z') { e.preventDefault(); undo(); }
    if (e.key === 'Escape') selectNone();
  }
  function reconcile(result: BatchResult) {
    const updates = new Map(result.previous.map((path, i) => [path, result.entries[i]]));
    entries = entries.map(e => updates.get(e.path) ?? e);
    selected = new Set([...selected].map(path => updates.get(path)?.path ?? path));
  }
  async function undo() {
    if (!native || !undoDepth || busy) return;
    busy = true;
    try {
      const result = await invoke<BatchResult>('undo_renames');
      reconcile(result);
      undoDepth = result.remaining;
      status = 'Undid one batch of ' + result.count + ' files';
    } catch (e) { status = 'Undo failed: ' + String(e); }
    finally { busy = false; }
  }
  let preview = $derived(plan(chosen, settings));
  let conflicts = $derived(preview.filter(x => !x.valid).length);
  let native = '__TAURI_INTERNALS__' in window;
  let changed = $derived(preview.filter(x => x.name !== x.target && x.valid).length);
  async function addPaths() {
    if (!native) { status = 'Browser mode: choose files for preview. Native renaming requires the desktop app.'; return; }
    try {
      const picked = await open({ multiple: true, directory: false });
      if (!picked) return;
      const paths = Array.isArray(picked) ? picked : [picked];
      append(paths.map(path => ({ path, name: path.split(/[\\/]/).pop() ?? path })));
      status = entries.length + ' files selected';
    } catch (e) { status = String(e); }
  }
  async function addFolder() {
    if (!native) { status = 'Folder selection for renaming requires the native app.'; return; }
    try {
      const picked = await open({ directory: true, multiple: false });
      if (!picked || Array.isArray(picked)) return;
      const rows = await invoke<Entry[]>('list_folder', { folder: picked });
      append(rows); status = rows.length + ' items found';
    } catch (e) { status = String(e); }
  }
  async function webFiles(ev: Event) {
    const input = ev.currentTarget as HTMLInputElement;
    const files = Array.from(input.files ?? []);
    append(files.map((f, i) => ({ path: 'browser/' + (f.webkitRelativePath || f.name) + '/' + i, name: f.name })));
    status = 'Browser preview only; no files modified'; input.value = '';
  }
  async function apply() {
    if (!native || busy || conflicts || !changed || !chosen.length || entries.some(x => x.path.startsWith('demo/'))) return;
    const changes = preview.filter(p => p.valid && p.name !== p.target).map(p => ({ from: p.path, to_name: p.target }));
    busy = true;
    try {
      const result = await invoke<BatchResult>('apply_renames', { changes });
      reconcile(result);
      undoDepth = result.remaining;
      status = 'Renamed ' + result.count + ' files in one undoable action';
    } catch (e) { status = 'Rename failed: ' + String(e); }
    finally { busy = false; }
  }
  function demo() { entries = [{ path: 'demo/Summer Holiday.jpg', name: 'Summer Holiday.jpg' }, { path: 'demo/Portrait Final.png', name: 'Portrait Final.png' }, { path: 'demo/Project Notes.pdf', name: 'Project Notes.pdf' }]; selected = new Set(entries.map(e => e.path)); status = 'Sample filenames loaded for preview'; }
  function shown(name: string, dir = false) { return settings.hideExtension ? splitName(name, dir)[0] : name; }
</script>
<svelte:head><title>Enumu — Batch Renamer</title><meta name="description" content="A beautifully simple, fast batch file renamer."/></svelte:head>
<svelte:window onkeydown={keys}/><div class="shell">
  <header class="topbar"><div class="brand"><div class="brandmark"><span>≡</span><i></i></div><span>enumu</span><span class="version">/ rename studio</span></div><div class="topright"><span class="live-dot"></span>{native ? 'Desktop workspace' : 'Web preview'}<span class="divider"></span><span class="privacy">● Local only</span></div></header>
  <main>
    <div class="intro"><div><div class="overline"><span class="overline-line"></span> FILE MANAGEMENT, REIMAGINED</div><h1>Rename with <em>precision.</em></h1><p>Every file, exactly how you want it. Simple controls. Instant preview.</p></div><div class="intro-badge"><span class="badge-icon">✦</span><div>YOUR WORKSPACE<small>Everything stays on your device</small></div></div></div>
    <div class="workspace">
      <section class="editor">
        <div class="section-title"><div class="title-left"><span class="section-index">01</span><div><h2>Rename Rules</h2><p>Define your transformation</p></div></div><span class="section-hint">LIVE PREVIEW <span class="live-dot"></span></span></div>
        <div class="editor-body">
          <div class="field-block"><div class="field-heading"><label>Action</label><small>What would you like to do?</small></div><div class="segment" role="group" aria-label="Action"><button class:active={settings.mode === 'add'} onclick={() => settings.mode = 'add'}><span class="symbol">＋</span> Add</button><button class:active={settings.mode === 'remove'} onclick={() => settings.mode = 'remove'}><span class="symbol">−</span> Remove</button></div></div>
          <div class="rule"></div>
          <div class="field-block"><div class="field-heading"><label>Content Type</label><small>Choose what to {settings.mode}</small></div><div class="segment" role="group" aria-label="Content Type"><button class:active={settings.kind === 'text'} onclick={() => settings.kind = 'text'}><span class="symbol">Aa</span> Text Box</button><button class:active={settings.kind === 'numbering'} onclick={() => settings.kind = 'numbering'}><span class="symbol">#</span> Numbering</button></div></div>
          {#if settings.kind === 'text'}
            <div class="field-block inset"><label for="content">Text {settings.mode === 'add' ? 'to Add' : 'to Remove'}</label><input id="content" type="text" bind:value={settings.text} placeholder={settings.mode === 'add' ? 'Enter text to insert…' : 'Enter text to find and remove…'} spellcheck="false" /><small class="input-help">{settings.mode === 'add' ? 'Inserted at your chosen character position.' : 'Removes the first matching text at or after your chosen position.'}</small></div>
          {:else}
            <div class="two-col inset"><div class="field-block"><label for="start">Start From</label><div class="input-wrap"><span class="input-icon">↗</span><input id="start" type="number" bind:value={settings.start} disabled={settings.mode === 'remove'} /></div></div><div class="field-block"><label for="digits"># Digits</label><div class="input-wrap"><span class="input-icon">#</span><input id="digits" type="number" min="1" max="12" bind:value={settings.digits}/></div></div></div>
          {/if}
          <div class="rule"></div>
          <div class="field-block"><div class="field-heading"><label>Position</label><small>Where should the change happen?</small></div><div class="position-line"><div class="input-wrap position-input"><span class="input-icon">⌗</span><input aria-label="Character position" type="number" min="0" bind:value={settings.position}/></div><div class="segment position-segment" role="group" aria-label="Position From"><button class:active={!settings.fromEnd} onclick={() => settings.fromEnd = false}>From Beginning</button><button class:active={settings.fromEnd} onclick={() => settings.fromEnd = true}>From End</button></div></div><small class="input-help">0 = the very {settings.fromEnd ? 'end' : 'beginning'} of the filename, excluding its extension.</small></div>
        </div>
        <div class="editor-foot"><span><span class="tiny-sparkle">✧</span> Non-destructive preview</span><button class="reset" onclick={() => settings = { mode: 'add', kind: 'text', text: '', position: 0, fromEnd: false, start: 1, digits: 2, hideExtension: true }}>↺ Reset Rules</button></div>
      </section>
      <section class="files">
        <div class="section-title"><div class="title-left"><span class="section-index">02</span><div><h2>File Queue <span class="count">{entries.length.toLocaleString()}</span></h2><p>Review before applying changes</p></div></div><button class="clear" onclick={() => { entries = []; selected = new Set(); status = 'Queue cleared'; }} disabled={!entries.length}>Clear All ×</button></div>
        <div class="queue-tools"><div class="queue-caption"><span class="live-dot"></span> {entries.length ? chosen.length + ' selected · ' + changed + ' changes' : 'Awaiting files'}</div><div class="selection-tools"><button onclick={selectAll} disabled={!entries.length}>All</button><button onclick={selectNone} disabled={!chosen.length}>None</button><button onclick={undo} disabled={!native || !undoDepth || busy}>↶ Undo {undoDepth}/100</button></div><label class="switch-label"><input type="checkbox" bind:checked={settings.hideExtension}/><span class="switch"></span> Hide extensions</label></div>
        {#if !entries.length}
          <div class="dropzone" class:dragover={dragOver} ondragover={(e) => { e.preventDefault(); dragOver = true; }} ondragleave={() => dragOver = false} ondrop={(e) => { e.preventDefault(); dragOver = false; status = 'Use Add Files to select files'; }}>
            <div class="upload-symbol"><span>↑</span></div><h3>Your files, your rules.</h3><p>Add files to see their new names instantly.</p>
            <div class="drop-actions">{#if native}<button class="primary" onclick={addPaths}>＋ Add Files</button><button class="secondary" onclick={addFolder}>▤ Add Folder</button>{:else}<label class="primary file-pick">＋ Choose Files<input type="file" multiple onchange={webFiles} hidden /></label><label class="secondary file-pick">▤ Choose Folder<input type="file" multiple webkitdirectory="" onchange={webFiles} hidden /></label>{/if}</div>
            <button class="sample" onclick={demo}>Try with sample files <span>↗</span></button>
          </div>
        {:else}
          <div class="queue-table"><div class="queue-header"><span><input aria-label="Select all" type="checkbox" checked={entries.length > 0 && selected.size === entries.length} onchange={e => e.currentTarget.checked ? selectAll() : selectNone()}/></span><span>ORIGINAL NAME</span><span>LIVE PREVIEW ↗</span></div><div class="queue-scroll">{#each entries as item, i (item.path)}{@const p = targets.get(item.path)}<div class="queue-row" class:chosen={selected.has(item.path)} role="row" tabindex="0" onclick={e => { if ((e.target as HTMLElement).tagName !== 'INPUT') selectRow(item.path,i,e.shiftKey,e.ctrlKey || e.metaKey); }} onkeydown={e => { if (e.key === ' ' || e.key === 'Enter') { e.preventDefault(); selectRow(item.path,i,e.shiftKey,e.ctrlKey || e.metaKey); } }}><span><input aria-label={'Select ' + item.name} type="checkbox" checked={selected.has(item.path)} onclick={e => e.stopPropagation()} onchange={e => checkRow(item.path,i,e.currentTarget.checked,(e as MouseEvent).shiftKey)}/></span><div class="filename"><span class="file-glyph">{item.directory ? '▤' : '▧'}</span><span title={item.name}>{shown(item.name,item.directory)}</span></div><div class="filename new" class:invalid={!!p && !p.valid}><span title={p?.target ?? item.name}>{p ? shown(p.target,item.directory) : '—'}</span>{#if p?.error}<span class="error">{p.error}</span>{/if}</div></div>{/each}</div></div>
          <div class="queue-add">{#if native}<button class="secondary" onclick={addPaths}>＋ Add Files</button><button class="secondary" onclick={addFolder}>▤ Add Folder</button>{:else}<label class="secondary file-pick">＋ Add Files<input type="file" multiple onchange={webFiles} hidden /></label><label class="secondary file-pick">▤ Add Folder<input type="file" multiple webkitdirectory="" onchange={webFiles} hidden /></label>{/if}</div>
        {/if}
        <div class="queue-footer"><div class="footer-meta"><span class="footer-status" class:error-status={conflicts > 0}>{conflicts ? conflicts + ' conflicts detected' : status}</span><small>{native ? 'Native file operations' : 'Preview mode · No files modified'}</small></div><button class="apply" disabled={!native || busy || !chosen.length || conflicts > 0 || changed === 0 || entries.some(x => x.path.startsWith('demo/'))} onclick={apply}>{native ? 'Apply Renames' : 'Preview Only'} <span>↗</span></button></div>
      </section>
    </div>
    <div class="bottom"><span>ENUMU <b>·</b> MADE FOR THE DETAILS</span><span>Fast by design <span class="bottom-star">✦</span> Private by default</span></div>
  </main>
</div>
<style>
:global(*){box-sizing:border-box}:global(html){background:#0a0d13}:global(body){margin:0;color:#e9edf5;font-family:Inter,ui-sans-serif,system-ui,-apple-system,BlinkMacSystemFont,"Segoe UI",sans-serif;-webkit-font-smoothing:antialiased}:global(button),:global(input){font:inherit}:global(button){cursor:pointer}:global(button:disabled){cursor:not-allowed;opacity:.4}.shell{min-height:100vh;background:radial-gradient(ellipse 70% 40% at 53% -10%,#1a2335 0%,transparent 70%),#0a0d13}.topbar{height:64px;border-bottom:1px solid #202632;display:flex;align-items:center;justify-content:space-between;padding:0 max(28px,calc((100vw - 1240px)/2));background:#0c1017c9}.brand{display:flex;align-items:center;gap:10px;font-size:20px;font-weight:760;letter-spacing:-.065em}.brandmark{width:28px;height:28px;background:#c3d0ff;border-radius:8px;color:#0b1021;display:grid;place-items:center;font-size:21px;font-weight:900;position:relative;line-height:1}.brandmark i{width:5px;height:5px;border-radius:50%;background:#6475d2;position:absolute;right:4px;top:5px}.version{font-size:11px;letter-spacing:0;color:#616d83;font-weight:500;margin-left:9px}.topright{display:flex;align-items:center;gap:9px;color:#8894a7;font-size:11px}.live-dot{width:6px;height:6px;border-radius:50%;display:inline-block;background:#7ed6ad;box-shadow:0 0 10px #7ed6ad66;flex-shrink:0}.divider{height:15px;width:1px;background:#343b49;margin:0 8px}.privacy{color:#a6b2c7}main{max-width:1240px;margin:auto;padding:56px 28px 25px}.intro{display:flex;justify-content:space-between;align-items:center;gap:24px;margin-bottom:37px}.overline{font-size:10px;letter-spacing:.18em;color:#8796b3;font-weight:700;display:flex;align-items:center;gap:10px}.overline-line{width:17px;height:1px;background:#99abff}h1{font-size:clamp(32px,3.3vw,46px);font-weight:660;letter-spacing:-.065em;line-height:1.15;margin:14px 0 10px}h1 em{font-style:normal;color:#a4b5ff}.intro p{font-size:13px;color:#8793a7;margin:0}.intro-badge{display:flex;align-items:center;gap:12px;color:#b4bed0;font-size:10px;font-weight:750;letter-spacing:.08em}.intro-badge small{display:block;color:#6f7b8f;font-size:11px;letter-spacing:0;font-weight:400;margin-top:5px}.badge-icon{font-size:20px;color:#8f9ee7}.workspace{display:grid;grid-template-columns:minmax(340px, .85fr) minmax(420px,1.15fr);gap:16px;align-items:start}.editor,.files{border:1px solid #293140;border-radius:15px;background:linear-gradient(160deg,#171d28,#111720 65%);box-shadow:0 12px 35px #0000001b;overflow:hidden}.section-title{height:82px;padding:19px 22px;display:flex;align-items:center;justify-content:space-between;border-bottom:1px solid #2b3240}.title-left{display:flex;gap:14px;align-items:center}.section-index{width:31px;height:31px;border:1px solid #343e51;border-radius:9px;display:grid;place-items:center;font-size:11px;color:#9faed0;font-weight:650;background:#222b3a}h2{margin:0;font-size:15px;font-weight:680;letter-spacing:-.025em}.section-title p{font-size:11px;color:#76849a;margin:5px 0 0}.section-hint{font-size:9px;color:#8594ac;letter-spacing:.1em;display:flex;align-items:center;gap:9px}.editor-body{padding:23px 22px 24px}.field-block{margin-bottom:0}.field-heading{display:flex;justify-content:space-between;gap:10px;align-items:baseline;margin-bottom:12px}.field-heading label,.field-block>label{font-size:12px;color:#d4dbea;font-weight:650}.field-heading small{font-size:10px;color:#78869d}.segment{display:flex;background:#0c111a;border:1px solid #303a4a;padding:4px;border-radius:11px;gap:4px}.segment button{border:0;background:transparent;color:#8592a9;flex:1;min-width:0;border-radius:7px;padding:11px 12px;font-size:12px;font-weight:600;transition:background .15s,color .15s,box-shadow .15s}.segment button.active{color:#e8edff;background:linear-gradient(120deg,#3d4d75,#2b395a);box-shadow:inset 0 0 0 1px #53668b88,0 2px 6px #0005}.segment button:not(.active):hover{color:#e1e8fa;background:#1c2535}.symbol{font-size:15px;margin-right:8px;color:#acbbef}.rule{height:1px;background:#29313e;margin:22px 0}.inset{margin-top:18px}.field-block input,.input-wrap{width:100%;border:1px solid #354053;background:#0d121b;color:#e8efff;border-radius:9px;height:42px;outline:0}.field-block>input{padding:0 13px;margin-top:10px;font-size:12px}.field-block input:focus,.input-wrap:focus-within{border-color:#8299dc;box-shadow:0 0 0 3px #6d84cf1b}.field-block input::placeholder{color:#58667c}.input-help{font-size:10px;color:#74839a;display:block;margin-top:9px;line-height:1.5}.two-col{display:grid;grid-template-columns:1fr 1fr;gap:13px}.two-col .field-block>label{display:block;margin-bottom:10px}.input-wrap{display:flex;align-items:center;padding:0 12px;gap:10px}.input-wrap input{border:0!important;box-shadow:none!important;background:transparent;height:100%;padding:0;font-size:13px;min-width:0}.input-icon{color:#8292b3;font-size:13px}.position-line{display:flex;gap:9px}.position-input{width:90px;flex-shrink:0}.position-segment{flex:1}.position-segment button{padding:10px 6px;font-size:11px}.editor-foot{border-top:1px solid #29313e;display:flex;justify-content:space-between;align-items:center;padding:14px 22px;color:#71819a;font-size:10px;gap:12px}.tiny-sparkle{color:#9daef3;font-size:15px;margin-right:5px}.reset,.clear{background:transparent;border:0;color:#9cabc6;font-size:11px;padding:7px}.reset:hover,.clear:hover{color:#fff}.count{font-size:10px;background:#2d3952;color:#c5d2ff;padding:3px 7px;border-radius:5px;margin-left:5px}.queue-tools{display:flex;align-items:center;justify-content:space-between;padding:15px 22px;border-bottom:1px solid #28303e;gap:8px}.queue-caption{display:flex;align-items:center;gap:9px;color:#8594ad;font-size:11px}.switch-label{display:flex;align-items:center;gap:9px;color:#9ba9c0;font-size:11px;cursor:pointer}.switch-label input{position:absolute;opacity:0}.switch{width:27px;height:16px;border-radius:30px;background:#364153;position:relative;transition:background .2s}.switch:after{content:"";position:absolute;width:10px;height:10px;border-radius:50%;background:#b7c1d4;top:3px;left:3px;transition:transform .2s}.switch-label input:checked+.switch{background:#657bcb}.switch-label input:checked+.switch:after{transform:translateX(11px);background:white}.dropzone{min-height:340px;display:flex;flex-direction:column;align-items:center;justify-content:center;text-align:center;margin:16px 20px;border:1px dashed #39475e;border-radius:11px;background:radial-gradient(circle at center,#1d29401a,transparent 75%);padding:28px 12px;transition:border-color .2s}.dropzone.dragover{border-color:#a2b2ff}.upload-symbol{height:53px;width:53px;border:1px solid #3d4b65;border-radius:16px;background:#202b40;display:grid;place-items:center;font-size:29px;color:#b4c4ff;margin-bottom:17px}.dropzone h3{font-size:16px;font-weight:670;letter-spacing:-.03em;margin:0 0 8px}.dropzone p{color:#77859c;font-size:11px;margin:0 0 21px}.drop-actions{display:flex;gap:8px;align-items:center}.primary,.secondary,.apply{display:inline-flex;justify-content:center;align-items:center;border-radius:8px;font-weight:700;font-size:11px;min-height:36px;padding:0 14px;text-decoration:none;cursor:pointer}.primary{border:1px solid #b4c4ff;background:#acbfff;color:#10172b}.primary:hover{background:#c5d1ff}.secondary{border:1px solid #3b4659;background:#222b3b;color:#c3cce0}.secondary:hover{background:#2d3950}.sample{margin-top:18px;background:transparent;border:0;color:#8d9ec5;font-size:11px}.sample span{margin-left:4px}.sample:hover{color:#c8d4ff}.queue-table{margin:15px 20px 0;border:1px solid #2d3748;border-radius:9px;overflow:hidden}.queue-header,.queue-row{display:grid;grid-template-columns:1fr 1fr;gap:12px;align-items:center}.queue-header{padding:12px;background:#202938;font-size:9px;color:#8e9cb3;letter-spacing:.1em;font-weight:700}.arrow{color:#8c9ff4}.queue-scroll{max-height:340px;overflow:auto}.queue-row{padding:12px;border-top:1px solid #2a3341;font-size:11px}.filename{display:flex;align-items:center;gap:8px;min-width:0;color:#b4bfd0}.filename>span:nth-child(2),.filename.new>span:first-child{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.file-glyph{color:#8698bb;font-size:16px}.filename.new{color:#b8c7ff;display:flex;flex-wrap:wrap}.filename.invalid,.error{color:#ffaaa9}.error{font-size:9px}.more{font-size:10px;padding:12px;color:#8a9ab0}.queue-add{padding:14px 20px;display:flex;gap:8px}.queue-footer{border-top:1px solid #2a3240;display:flex;justify-content:space-between;align-items:center;gap:12px;padding:15px 20px}.footer-meta{display:flex;flex-direction:column;gap:5px;min-width:0}.footer-status{font-size:11px;color:#9daac2;overflow-wrap:anywhere}.footer-meta small{font-size:10px;color:#66758d}.error-status{color:#ffaaa9}.apply{background:#a7baff;color:#10172b;border:1px solid #bdcaff;gap:18px;white-space:nowrap}.apply span{font-size:15px}.apply:hover:not(:disabled){background:#c2d0ff}.bottom{display:flex;justify-content:space-between;color:#59667c;font-size:9px;letter-spacing:.13em;margin-top:28px}.bottom b{color:#94a5dc;margin:0 5px}.bottom-star{color:#9daeed;margin:0 8px}@media(max-width:940px){.workspace{grid-template-columns:1fr}.intro-badge{display:none}.dropzone{min-height:290px}}@media(max-width:600px){main{padding:35px 14px}.topbar{padding:0 16px;height:58px}.version,.privacy,.topright .divider{display:none}.intro{margin-bottom:27px}.intro p{line-height:1.6}.section-title{padding:16px;height:76px}.editor-body{padding:19px 16px}.editor-foot,.queue-tools,.queue-footer{padding-left:16px;padding-right:16px}.position-line{flex-wrap:wrap}.position-segment{min-width:230px}.section-hint{display:none}.bottom{gap:12px;line-height:1.7}.dropzone{margin:12px}.queue-table{margin:12px}.queue-row{gap:7px;padding:10px}.queue-header{gap:7px}.queue-footer{flex-wrap:wrap}.apply{margin-left:auto}}
.selection-tools{display:flex;gap:5px;align-items:center;flex-wrap:wrap}.selection-tools button{border:1px solid #3a465c;border-radius:6px;background:#202b3c;color:#c0ceea;padding:6px 8px;font-size:10px}.queue-header,.queue-row{grid-template-columns:24px minmax(0,1fr) minmax(0,1fr)}.queue-header input,.queue-row input{accent-color:#a9baff;cursor:pointer}.queue-row{cursor:pointer;user-select:none}.queue-row.chosen{background:#25344b99}.queue-row:focus-visible{outline:2px solid #aabaff;outline-offset:-2px}.queue-scroll{max-height:440px}.queue-tools{flex-wrap:wrap}.drop-actions{flex-wrap:wrap;justify-content:center}@media(max-width:600px){.queue-header,.queue-row{grid-template-columns:20px minmax(0,1fr) minmax(0,1fr)}}
</style>
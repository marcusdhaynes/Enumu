<script lang="ts">
  // Permanent Updates section (DORMANT — see updates/UPDATES_SPEC.md).
  // When a Settings menu exists, render this at the very BOTTOM of Settings.
  import { onMount } from 'svelte';
  import { checkForUpdate, applyUpdate, CURRENT_VERSION, type UpdateInfo } from './updateCheck';
  let update: UpdateInfo | null = $state(null);
  let checking = $state(false);
  let checked = $state(false);
  async function check() { checking = true; try { update = await checkForUpdate(); } catch { update = null; } finally { checking = false; checked = true; } }
  onMount(check);
</script>

<section aria-label="Updates" style="border-top:1px solid rgba(127,127,127,0.3);padding-top:16px;margin-top:16px;">
  <h3 style="margin:0 0 8px;font-size:14px;">Updates</h3>
  <div style="font-size:13px;display:flex;flex-direction:column;gap:4px;">
    <div>Current version: <strong>{CURRENT_VERSION === 'unknown' ? 'Unknown' : `v${CURRENT_VERSION}`}</strong></div>
    <div>Available version: <strong>{update ? `v${update.version}` : checked && !checking ? 'None — you are up to date' : '—'}</strong></div>
  </div>
  {#if update?.notes}
    <div style="margin-top:8px;"><div style="font-size:13px;font-weight:600;">What's New</div>
    <pre style="white-space:pre-wrap;font-size:12px;margin:4px 0 0;font-family:inherit;">{update.notes}</pre></div>
  {/if}
  <div style="display:flex;gap:8px;margin-top:12px;">
    <button type="button" onclick={check} disabled={checking}>{checking ? 'Checking…' : 'Check for updates'}</button>
    {#if update}<button type="button" onclick={() => update && applyUpdate(update)}>Update to v{update.version}</button>{/if}
  </div>
</section>

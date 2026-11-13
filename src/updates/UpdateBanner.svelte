<script lang="ts">
  // Uniform update ribbon (DORMANT — see updates/UPDATES_SPEC.md).
  // Thin, top-center, neutral/theme-aware. The ribbon itself is the button
  // (click = update). The × dismisses it permanently for that version
  // (localStorage `updates:dismissed:<version>`). No Update/Later buttons.
  import { onMount } from 'svelte';
  import { checkForUpdate, applyUpdate, dismissVersion, isDismissed, type UpdateInfo } from './updateCheck';
  let update: UpdateInfo | null = $state(null);
  let hidden = $state(false);
  onMount(async () => { update = await checkForUpdate().catch(() => null); });
</script>

{#if update && !hidden && !isDismissed(update.version)}
  <div style="position:fixed;top:8px;left:0;right:0;z-index:9999;display:flex;justify-content:center;pointer-events:none;">
    <div role="button" tabindex="0" title="Click to update"
      onclick={() => update && applyUpdate(update)}
      onkeydown={(e) => { if ((e.key === 'Enter' || e.key === ' ') && update) { e.preventDefault(); applyUpdate(update); } }}
      style="pointer-events:auto;display:flex;align-items:center;gap:8px;height:24px;padding:0 6px 0 12px;border-radius:999px;font-size:12px;line-height:24px;cursor:pointer;user-select:none;background:rgba(127,127,127,0.16);border:1px solid rgba(127,127,127,0.35);backdrop-filter:blur(6px);">
      <span>Update available · v{update.version}</span>
      <button type="button" aria-label="Dismiss this update"
        onclick={(e) => { e.stopPropagation(); if (update) dismissVersion(update.version); hidden = true; }}
        style="pointer-events:auto;border:none;background:transparent;color:inherit;cursor:pointer;font-size:14px;line-height:1;padding:2px 4px;opacity:0.7;">×</button>
    </div>
  </div>
{/if}

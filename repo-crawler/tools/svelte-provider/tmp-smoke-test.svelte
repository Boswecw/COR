<script lang="ts">
  let count = $state(0);
  const doubled = $derived(count * 2);
</script>

<button onclick={() => count++}>
  {count} / {doubled}
</button>

<script setup lang="ts">
defineProps<{
  collapsed: boolean;
  title: string;
}>();

const emit = defineEmits<{
  toggle: [];
}>();
</script>

<template>
  <button
    type="button"
    class="list-header collapsible-header"
    :aria-expanded="!collapsed"
    @click="emit('toggle')"
  >
    <h2>{{ title }}</h2>
    <span class="collapse-button" :class="{ collapsed }" aria-hidden="true">▾</span>
  </button>

  <slot v-if="!collapsed"></slot>
</template>

<style scoped>
.list-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  width: 100%;
  min-height: 40px;
  margin-top: 6px;
  padding: 7px 8px;
  color: inherit;
  text-align: left;
  border: 0;
  border-radius: var(--radius);
  background: transparent;
  transition:
    background-color 0.16s ease,
    color 0.16s ease;
}

.list-header:first-child {
  margin-top: 0;
}

.list-header:hover:not(:disabled) {
  color: var(--accent-strong);
  background: rgba(237, 244, 255, 0.92);
}

.list-header:focus-visible {
  outline: 3px solid rgba(39, 100, 216, 0.16);
}

.list-header h2 {
  margin: 0;
  color: var(--text);
  font-size: 15px;
  line-height: 1.3;
  letter-spacing: 0;
}
</style>

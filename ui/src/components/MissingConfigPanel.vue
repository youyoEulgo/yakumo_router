<script setup lang="ts">
defineProps<{
  creating: boolean;
  messages: {
    action: string;
    creating: string;
    detail: string;
    title: string;
  };
}>();

const emit = defineEmits<{
  create: [];
}>();
</script>

<template>
  <section class="missing-config-panel">
    <div>
      <h2>{{ messages.title }}</h2>
      <p>{{ messages.detail }}</p>
    </div>
    <button class="primary-button" type="button" :disabled="creating" @click="emit('create')">
      {{ creating ? messages.creating : messages.action }}
    </button>
  </section>
</template>

<style scoped>
.missing-config-panel {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 18px;
  padding: 18px 20px;
  border: 1px solid var(--accent-border);
  border-radius: var(--radius);
  background: var(--accent-soft);
  box-shadow: var(--shadow-sm);
}

.missing-config-panel h2,
.missing-config-panel p {
  margin: 0;
}

.missing-config-panel h2 {
  color: var(--text);
  font-size: 15px;
  line-height: 1.3;
}

.missing-config-panel p {
  margin-top: 6px;
  color: var(--text-muted);
  font-size: 13px;
  line-height: 1.45;
}

@media (max-width: 760px) {
  .missing-config-panel {
    align-items: stretch;
    flex-direction: column;
  }
}
</style>

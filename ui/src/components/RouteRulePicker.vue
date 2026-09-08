<script setup lang="ts">
import { ref } from 'vue';
import { useI18n } from '../i18n';
import type { RouteRule } from '../types';

const props = defineProps<{
  disabled: boolean;
  routes: RouteRule[];
}>();

const emit = defineEmits<{
  add: [ids: string[]];
  cancel: [];
}>();

const { t } = useI18n();
const selected = ref<string[]>([]);

function toggle(id: string, checked: boolean): void {
  if (checked) {
    if (!selected.value.includes(id)) {
      selected.value = [...selected.value, id];
    }
    return;
  }

  selected.value = selected.value.filter((item) => item !== id);
}

function submit(): void {
  if (props.disabled || selected.value.length === 0) {
    return;
  }

  emit('add', selected.value);
}
</script>

<template>
  <div class="rule-picker">
    <div v-if="routes.length === 0" class="empty-state">{{ t('noAvailableRules') }}</div>
    <template v-else>
      <label v-for="route in routes" :key="route.id" class="rule-picker-row">
        <input
          type="checkbox"
          :checked="selected.includes(route.id)"
          :disabled="disabled"
          @change="toggle(route.id, ($event.target as HTMLInputElement).checked)"
        />
        <span class="rule-picker-text">
          <strong>{{ route.id }}</strong>
          <small>
            {{ route.match_type ?? 'contains' }} {{ route.match }} / {{ route.provider }}
          </small>
        </span>
      </label>

      <div class="actions rule-picker-actions">
        <button
          class="primary-button compact"
          type="button"
          :disabled="disabled || selected.length === 0"
          @click="submit"
        >
          {{ t('addSelectedRules', { count: selected.length }) }}
        </button>
        <button
          class="ghost-button compact"
          type="button"
          :disabled="disabled"
          @click="emit('cancel')"
        >
          {{ t('cancel') }}
        </button>
      </div>
    </template>
  </div>
</template>

<style scoped>
.rule-picker {
  display: grid;
  gap: 8px;
  padding: 12px;
  border: 1px dashed var(--border-strong);
  border-radius: var(--radius);
  background: var(--picker-bg);
}

.rule-picker-row {
  display: grid;
  grid-template-columns: 18px minmax(0, 1fr);
  gap: 10px;
  align-items: center;
  padding: 8px 10px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface);
  cursor: pointer;
}

.rule-picker-row:hover {
  border-color: var(--accent-border);
}

.rule-picker-text {
  display: grid;
  gap: 3px;
  min-width: 0;
}

.rule-picker-text strong,
.rule-picker-text small {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.rule-picker-text strong {
  color: var(--text);
  font-size: 13px;
}

.rule-picker-text small {
  color: var(--text-muted);
  font-size: 12px;
}

.rule-picker-actions {
  padding-top: 2px;
}
</style>

<script setup lang="ts">
import SaveIcon from './SaveIcon.vue';
import { useI18n } from '../i18n';
import type { RouteRule } from '../types';

defineProps<{
  isEditingRoute: boolean;
  routeForm: RouteRule;
  savingRoute: boolean;
}>();

const emit = defineEmits<{
  deleteRoute: [];
  saveRoute: [];
  updateRouteField: [field: keyof RouteRule, value: boolean | string];
}>();

const { t } = useI18n();

function updateRouteTextField(field: keyof RouteRule, event: Event): void {
  emit(
    'updateRouteField',
    field,
    (event.target as HTMLInputElement | HTMLSelectElement).value.trim(),
  );
}

function updateForwardOnly(event: Event): void {
  emit('updateRouteField', 'forward_only', (event.target as HTMLInputElement).checked);
}
</script>

<template>
  <form class="form-grid" @submit.prevent="emit('saveRoute')">
    <label>
      <span>ID</span>
      <input
        :value="routeForm.id"
        required
        autocomplete="off"
        placeholder="openai-gpt"
        @input="updateRouteTextField('id', $event)"
      />
    </label>

    <label>
      <span>{{ t('match') }}</span>
      <input
        :value="routeForm.match"
        required
        autocomplete="off"
        placeholder="gpt"
        @input="updateRouteTextField('match', $event)"
      />
    </label>

    <label>
      <span>{{ t('matchType') }}</span>
      <select :value="routeForm.match_type" @change="updateRouteTextField('match_type', $event)">
        <option value="contains">{{ t('contains') }}</option>
        <option value="exact">{{ t('exact') }}</option>
        <option value="regex">{{ t('regex') }}</option>
      </select>
    </label>

    <label>
      <span>{{ t('upstreamModel') }}</span>
      <input
        :value="routeForm.model"
        :required="!routeForm.forward_only"
        autocomplete="off"
        :disabled="routeForm.forward_only"
        placeholder="openai/gpt-4.1"
        @input="updateRouteTextField('model', $event)"
      />
    </label>

    <label class="checkbox-row">
      <input :checked="routeForm.forward_only" type="checkbox" @change="updateForwardOnly" />
      <span>{{ t('forwardOnly') }}</span>
    </label>

    <div class="actions">
      <button class="primary-button" type="submit" :disabled="savingRoute">
        <SaveIcon />
        {{ savingRoute ? t('saving') : t('save') }}
      </button>
      <button
        class="danger-button"
        type="button"
        :disabled="!isEditingRoute || savingRoute"
        @click="emit('deleteRoute')"
      >
        {{ t('delete') }}
      </button>
    </div>
  </form>
</template>

<style scoped>
.form-grid {
  display: grid;
  max-width: 720px;
  gap: 16px;
}

.form-grid label {
  display: grid;
  gap: 7px;
  color: #425066;
  font-size: 12px;
  font-weight: 700;
}

.form-grid input,
.form-grid select {
  width: 100%;
  min-height: 42px;
  padding: 0 12px;
  color: var(--text);
  border: 1px solid var(--border-strong);
  border-radius: var(--radius);
  background: var(--surface);
  box-shadow: inset 0 1px 0 rgba(18, 24, 38, 0.03);
  transition:
    border-color 0.16s ease,
    box-shadow 0.16s ease,
    background-color 0.16s ease;
}

.form-grid input:disabled {
  color: var(--text-soft);
  background: #eef2f6;
}

.form-grid input:focus,
.form-grid select:focus {
  border-color: var(--accent);
  outline: 3px solid rgba(39, 100, 216, 0.16);
}

.checkbox-row {
  grid-template-columns: auto 1fr;
  align-items: center;
}

.checkbox-row input {
  width: 18px;
  min-height: 18px;
  padding: 0;
  accent-color: var(--accent);
}
</style>

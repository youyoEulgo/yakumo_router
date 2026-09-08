<script setup lang="ts">
import { computed } from 'vue';
import SaveIcon from './SaveIcon.vue';
import { useI18n } from '../i18n';
import type { RouteRule } from '../types';

const props = defineProps<{
  isEditingRoute: boolean;
  routeForm: RouteRule;
  savingRoute: boolean;
}>();

const emit = defineEmits<{
  deleteRoute: [];
  renameRoute: [];
  saveRoute: [];
  updateRouteField: [field: keyof RouteRule, value: boolean | string];
}>();

const { t } = useI18n();

const matchPlaceholder = computed(() => {
  switch (props.routeForm.match_type ?? 'contains') {
    case 'exact':
      return 'claude-3-5-sonnet';
    case 'regex':
      return '^claude-.*-(opus|sonnet)$';
    default:
      return 'gpt';
  }
});

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
      <div class="field-row">
        <input
          :value="routeForm.id"
          required
          autocomplete="off"
          :disabled="isEditingRoute"
          placeholder="openai-gpt"
          @input="updateRouteTextField('id', $event)"
        />
        <button
          v-if="isEditingRoute"
          class="ghost-button compact"
          type="button"
          :disabled="savingRoute"
          @click="emit('renameRoute')"
        >
          {{ t('rename') }}
        </button>
      </div>
    </label>

    <label>
      <span>{{ t('match') }}</span>
      <input
        :value="routeForm.match"
        required
        autocomplete="off"
        :placeholder="matchPlaceholder"
        @input="updateRouteTextField('match', $event)"
      />
      <small v-if="routeForm.match_type === 'regex'" class="field-hint">
        {{ t('regexFullMatchHint') }}
      </small>
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
.field-hint {
  color: var(--text-muted);
  font-size: 11px;
  font-weight: 500;
  line-height: 1.4;
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

<script setup lang="ts">
import SaveIcon from './SaveIcon.vue';
import { useI18n } from '../i18n';

defineProps<{
  deletingProvider: boolean;
  isEditingProvider: boolean;
  providerForm: {
    name: string;
    base_url: string;
    api_key: string;
  };
  savingProvider: boolean;
}>();

const showApiKey = defineModel<boolean>('showApiKey', { required: true });
const { t } = useI18n();

const emit = defineEmits<{
  deleteProvider: [];
  renameProvider: [];
  saveProvider: [];
  updateProviderField: [field: 'name' | 'base_url' | 'api_key', value: string];
}>();

function updateProviderField(field: 'name' | 'base_url' | 'api_key', event: Event): void {
  emit('updateProviderField', field, (event.target as HTMLInputElement).value.trim());
}
</script>

<template>
  <form class="form-grid" @submit.prevent="emit('saveProvider')">
    <label>
      <span>{{ t('name') }}</span>
      <div class="field-row">
        <input
          :value="providerForm.name"
          required
          autocomplete="off"
          :disabled="isEditingProvider"
          placeholder="openrouter"
          @input="updateProviderField('name', $event)"
        />
        <button
          v-if="isEditingProvider"
          class="ghost-button compact"
          type="button"
          :disabled="savingProvider"
          @click="emit('renameProvider')"
        >
          {{ t('rename') }}
        </button>
      </div>
    </label>

    <label>
      <span>{{ t('baseUrl') }}</span>
      <input
        :value="providerForm.base_url"
        required
        autocomplete="off"
        placeholder="https://openrouter.ai/api/v1"
        @input="updateProviderField('base_url', $event)"
      />
    </label>

    <label>
      <span>{{ t('apiKey') }}</span>
      <span class="field-row">
        <input
          :value="providerForm.api_key"
          required
          autocomplete="off"
          placeholder="sk-..."
          :type="showApiKey ? 'text' : 'password'"
          @input="updateProviderField('api_key', $event)"
        />
        <button
          class="icon-button"
          type="button"
          :aria-label="showApiKey ? t('hideApiKey') : t('showApiKey')"
          :title="showApiKey ? t('hideApiKey') : t('showApiKey')"
          @click="showApiKey = !showApiKey"
        >
          <svg
            v-if="showApiKey"
            aria-hidden="true"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path
              d="M17.94 17.94A10.94 10.94 0 0 1 12 20C7 20 2.73 16.89 1 12a11.74 11.74 0 0 1 5.06-5.94"
            />
            <path d="M10.59 10.59a2 2 0 0 0 2.82 2.82" />
            <path d="m3 3 18 18" />
            <path d="M14.12 5.14A10.93 10.93 0 0 1 23 12a11.76 11.76 0 0 1-2.27 3.46" />
          </svg>
          <svg
            v-else
            aria-hidden="true"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path d="M1 12s4-7 11-7 11 7 11 7-4 7-11 7-11-7-11-7Z" />
            <circle cx="12" cy="12" r="3" />
          </svg>
        </button>
      </span>
    </label>

    <div class="actions">
      <button class="primary-button" type="submit" :disabled="savingProvider">
        <SaveIcon />
        {{ savingProvider ? t('saving') : t('save') }}
      </button>
      <button
        class="danger-button"
        type="button"
        :disabled="!isEditingProvider || deletingProvider"
        @click="emit('deleteProvider')"
      >
        {{ deletingProvider ? t('deleting') : t('delete') }}
      </button>
    </div>
  </form>
</template>

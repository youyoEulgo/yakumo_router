<script setup lang="ts">
import SaveIcon from './SaveIcon.vue';
import { useI18n } from '../i18n';

defineProps<{
  deleting: boolean;
  routeTableName: string;
  saving: boolean;
  selectedRouteTable: string;
}>();

const emit = defineEmits<{
  delete: [];
  renameRouteTable: [];
  save: [];
  'update:routeTableName': [name: string];
}>();

const { t } = useI18n();

function updateRouteTableName(event: Event): void {
  emit('update:routeTableName', (event.target as HTMLInputElement).value);
}
</script>

<template>
  <form class="form-grid" @submit.prevent="emit('save')">
    <label>
      <span>{{ t('name') }}</span>
      <div class="field-row">
        <input
          :value="routeTableName"
          required
          autocomplete="off"
          :disabled="Boolean(selectedRouteTable)"
          placeholder="default"
          @input="updateRouteTableName"
        />
        <button
          v-if="selectedRouteTable"
          class="ghost-button compact"
          type="button"
          :disabled="saving"
          @click="emit('renameRouteTable')"
        >
          {{ t('rename') }}
        </button>
      </div>
    </label>

    <div class="actions">
      <button class="primary-button" type="submit" :disabled="saving">
        <SaveIcon />
        {{ saving ? t('saving') : t('save') }}
      </button>
      <button
        class="danger-button"
        type="button"
        :disabled="!selectedRouteTable || deleting"
        @click="emit('delete')"
      >
        {{ deleting ? t('deleting') : t('delete') }}
      </button>
    </div>
  </form>
</template>

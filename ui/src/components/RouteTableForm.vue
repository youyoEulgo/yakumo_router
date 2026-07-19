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
  save: [];
  'update:routeTableName': [name: string];
}>();

const { t } = useI18n();

function updateRouteTableName(event: Event): void {
  emit('update:routeTableName', (event.target as HTMLInputElement).value);
}
</script>

<template>
  <form class="route-table-form" @submit.prevent="emit('save')">
    <label>
      <span>{{ t('name') }}</span>
      <input
        :value="routeTableName"
        required
        autocomplete="off"
        placeholder="default"
        @input="updateRouteTableName"
      />
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

<style scoped>
.route-table-form {
  display: grid;
  max-width: 720px;
  gap: 16px;
}

.route-table-form label {
  display: grid;
  gap: 7px;
  color: #425066;
  font-size: 12px;
  font-weight: 700;
}

.route-table-form input {
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

.route-table-form input:focus {
  border-color: var(--accent);
  outline: 3px solid rgba(39, 100, 216, 0.16);
}

@media (max-width: 900px) {
  .route-table-form {
    grid-template-columns: 1fr;
  }
}
</style>

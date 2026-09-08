<script setup lang="ts">
import { ref } from 'vue';
import ProviderForm from './ProviderForm.vue';
import RenameDialog from './RenameDialog.vue';
import RouteRuleForm from './RouteRuleForm.vue';
import { protocolLabel, useI18n } from '../i18n';
import type { Protocol, RouteRule } from '../types';

defineProps<{
  activeProtocol: Protocol;
  deletingProvider: boolean;
  isEditingProvider: boolean;
  isEditingRoute: boolean;
  providerForm: {
    name: string;
    base_url: string;
    api_key: string;
  };
  providerRoutes: RouteRule[];
  renameProviderDialogOpen: boolean;
  renameRouteDialogOpen: boolean;
  renamingProvider: boolean;
  renamingRoute: boolean;
  routeEditorOpen: boolean;
  routeForm: RouteRule;
  savingProvider: boolean;
  savingRoute: boolean;
  selectedProvider: string;
  selectedRouteId: string;
}>();

const showApiKey = defineModel<boolean>('showApiKey', { required: true });
const rulesCollapsed = ref(false);
const { t } = useI18n();

const emit = defineEmits<{
  updateProviderField: [field: 'name' | 'base_url' | 'api_key', value: string];
  updateRouteField: [field: keyof RouteRule, value: boolean | string];
  saveProvider: [];
  deleteProvider: [];
  resetRoute: [];
  selectRoute: [route: RouteRule];
  saveRoute: [];
  deleteRoute: [];
  openRenameProviderDialog: [];
  closeRenameProviderDialog: [];
  renameProvider: [name: string];
  openRenameRouteDialog: [];
  closeRenameRouteDialog: [];
  renameRoute: [id: string];
}>();
</script>

<template>
  <section class="panel">
    <div class="panel-header">
      <div>
        <h2>{{ isEditingProvider ? t('editProvider') : t('newProvider') }}</h2>
        <p class="panel-note">
          {{ t('providerSettings', { protocol: protocolLabel(activeProtocol) }) }}
        </p>
      </div>
    </div>

    <ProviderForm
      v-model:show-api-key="showApiKey"
      :deleting-provider="deletingProvider"
      :is-editing-provider="isEditingProvider"
      :provider-form="providerForm"
      :saving-provider="savingProvider"
      @save-provider="emit('saveProvider')"
      @delete-provider="emit('deleteProvider')"
      @rename-provider="emit('openRenameProviderDialog')"
      @update-provider-field="(field, value) => emit('updateProviderField', field, value)"
    />
  </section>

  <section class="panel" :class="{ muted: !selectedProvider }">
    <button
      class="panel-header collapsible-header"
      type="button"
      :disabled="!selectedProvider"
      :aria-expanded="selectedProvider ? !rulesCollapsed : false"
      @click="rulesCollapsed = !rulesCollapsed"
    >
      <div>
        <h2>{{ t('rules') }}</h2>
        <p v-if="selectedProvider" class="panel-note">
          {{ t('routesUsing', { provider: selectedProvider }) }}
        </p>
        <p v-else class="panel-note">{{ t('selectProviderForRules') }}</p>
      </div>
      <span class="collapse-button" :class="{ collapsed: rulesCollapsed }" aria-hidden="true">
        ▾
      </span>
    </button>

    <div v-if="selectedProvider && !rulesCollapsed" class="rules-layout">
      <div class="rule-list">
        <div v-if="providerRoutes.length === 0" class="empty-state">
          {{ t('noProviderRules') }}
        </div>
        <button
          v-for="route in providerRoutes"
          :key="route.id"
          type="button"
          class="card-row route-row"
          :class="{ selected: selectedRouteId === route.id }"
          @click="emit('selectRoute', route)"
        >
          <span class="route-id">{{ route.id }}</span>
          <span class="route-detail">
            {{ route.match_type ?? 'contains' }} {{ route.match }}
            {{ route.forward_only ? `-> ${t('forwardOnly')}` : `-> ${route.model}` }}
          </span>
        </button>
        <button
          type="button"
          class="card-row route-row new-row"
          :aria-label="t('newRule')"
          @click="emit('resetRoute')"
        >
          <span class="new-row-plus" aria-hidden="true">+</span>
        </button>
      </div>

      <RouteRuleForm
        v-if="routeEditorOpen"
        :is-editing-route="isEditingRoute"
        :route-form="routeForm"
        :saving-route="savingRoute"
        @save-route="emit('saveRoute')"
        @delete-route="emit('deleteRoute')"
        @rename-route="emit('openRenameRouteDialog')"
        @update-route-field="(field, value) => emit('updateRouteField', field, value)"
      />
    </div>
  </section>

  <RenameDialog
    v-if="renameProviderDialogOpen"
    :current-value="providerForm.name"
    :label="t('newName')"
    :note="t('renameProviderNote', { name: providerForm.name })"
    :saving="renamingProvider"
    :title="t('renameProvider')"
    @cancel="emit('closeRenameProviderDialog')"
    @confirm="(name) => emit('renameProvider', name)"
  />

  <RenameDialog
    v-if="renameRouteDialogOpen"
    :current-value="routeForm.id"
    :label="t('newId')"
    :note="t('renameRuleNote', { id: routeForm.id })"
    :saving="renamingRoute"
    :title="t('renameRule')"
    @cancel="emit('closeRenameRouteDialog')"
    @confirm="(id) => emit('renameRoute', id)"
  />
</template>

<style scoped>
.rules-layout {
  display: grid;
  grid-template-columns: minmax(220px, 320px) minmax(0, 1fr);
  align-items: start;
  gap: 18px;
}

.rule-list {
  display: grid;
  align-content: start;
  gap: 10px;
}

.route-row {
  min-height: 72px;
}

.route-id {
  min-width: 0;
  overflow: hidden;
  color: var(--text);
  font-size: 14px;
  font-weight: 700;
  line-height: 1.3;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.route-detail {
  overflow: hidden;
  color: var(--text-muted);
  font-size: 12px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

@media (max-width: 900px) {
  .rules-layout {
    grid-template-columns: 1fr;
  }
}
</style>

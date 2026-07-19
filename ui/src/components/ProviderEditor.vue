<script setup lang="ts">
import { ref } from 'vue';
import ProviderForm from './ProviderForm.vue';
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
}>();
</script>

<template>
  <section class="panel">
    <div class="panel-header">
      <div>
        <h2>{{ isEditingProvider ? t('editProvider') : t('newProvider') }}</h2>
        <p>{{ t('providerSettings', { protocol: protocolLabel(activeProtocol) }) }}</p>
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
        <p v-if="selectedProvider">{{ t('routesUsing', { provider: selectedProvider }) }}</p>
        <p v-else>{{ t('selectProviderForRules') }}</p>
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
          class="route-row"
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
          class="route-row new-row"
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
        @update-route-field="(field, value) => emit('updateRouteField', field, value)"
      />
    </div>
  </section>
</template>

<style scoped>
.panel {
  display: grid;
  gap: 18px;
  padding: 20px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background:
    linear-gradient(180deg, rgba(255, 255, 255, 0.9), rgba(249, 251, 252, 0.86)), var(--surface);
  box-shadow:
    0 1px 0 rgba(255, 255, 255, 0.7) inset,
    var(--shadow-sm);
}

.panel.muted {
  opacity: 0.72;
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.panel-header h2,
.panel-header p {
  margin: 0;
}

.panel-header h2 {
  color: var(--text);
  font-size: 15px;
  line-height: 1.3;
  letter-spacing: 0;
}

.panel-header p {
  margin-top: 6px;
  color: var(--text-muted);
  font-size: 13px;
}

.collapsible-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  width: 100%;
  min-height: 40px;
  padding: 7px 8px;
  color: inherit;
  line-height: 1;
  text-align: left;
  border: 0;
  background: transparent;
  transition:
    background-color 0.16s ease,
    color 0.16s ease;
}

.collapsible-header:hover:not(:disabled) {
  color: var(--accent-strong);
  background: rgba(237, 244, 255, 0.92);
}

.collapsible-header:focus-visible {
  outline: 3px solid rgba(39, 100, 216, 0.16);
}

.collapsible-header:disabled {
  cursor: default;
  opacity: 1;
}

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
  position: relative;
  display: grid;
  gap: 6px;
  width: 100%;
  min-height: 72px;
  padding: 12px 13px;
  text-align: left;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background:
    linear-gradient(180deg, rgba(255, 255, 255, 0.98), rgba(250, 252, 253, 0.94)),
    var(--surface-raised);
  box-shadow: var(--shadow-sm);
  transition:
    border-color 0.16s ease,
    background-color 0.16s ease,
    box-shadow 0.16s ease,
    transform 0.16s ease;
}

.route-row:hover {
  border-color: var(--accent-border);
  background: var(--surface);
  box-shadow: var(--shadow-md);
  transform: translateY(-1px);
}

.route-row.selected {
  border-color: var(--accent);
  background: var(--accent-soft);
  box-shadow:
    inset 4px 0 0 var(--accent),
    var(--shadow-sm);
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

.new-row {
  place-items: center;
  min-height: 56px;
  color: var(--text-muted);
  border-style: dashed;
  background: rgba(255, 255, 255, 0.6);
  box-shadow: none;
}

.new-row:hover {
  color: var(--accent);
  border-color: var(--accent-border);
  background: var(--accent-soft);
}

.new-row-plus {
  display: grid;
  place-items: center;
  width: 28px;
  height: 28px;
  border: 1px solid currentColor;
  border-radius: 50%;
  font-size: 22px;
  font-weight: 600;
  line-height: 1;
}

@media (max-width: 900px) {
  .rules-layout {
    grid-template-columns: 1fr;
  }
}
</style>

<script setup lang="ts">
import { reactive } from 'vue';
import { useRouteDragSort } from '../composables/useRouteDragSort';
import { protocolLabel, useI18n } from '../i18n';
import type { Protocol, RouteRule, RouteTable, RouteTableEntry } from '../types';
import RouteRulePicker from './RouteRulePicker.vue';
import RouteTableForm from './RouteTableForm.vue';
import RouteToggleRow from './RouteToggleRow.vue';

const props = defineProps<{
  activating: boolean;
  activeRouteTable: string | null;
  deleting: boolean;
  routeTable: RouteTable | undefined;
  routeTableName: string;
  routes: Record<Protocol, RouteRule[]>;
  saving: boolean;
  selectedRouteTable: string;
}>();

const emit = defineEmits<{
  'update:routeTableName': [name: string];
  activate: [];
  addRoutes: [protocol: Protocol, ids: string[]];
  delete: [];
  moveRoute: [protocol: Protocol, routeId: string, direction: -1 | 1];
  removeRoute: [protocol: Protocol, routeId: string];
  save: [];
  toggleRoute: [protocol: Protocol, routeId: string, enabled: boolean];
}>();

const { t } = useI18n();

const pickerOpen = reactive<Record<Protocol, boolean>>({
  openai: false,
  anthropic: false,
});

type VisibleRoute = {
  entry: RouteTableEntry;
  route: RouteRule;
};

function visibleRoutes(protocol: Protocol): VisibleRoute[] {
  const entries = props.routeTable?.[protocol] ?? [];

  return entries
    .map((entry) => ({
      entry,
      route: props.routes[protocol].find((route) => route.id === entry.id),
    }))
    .filter((item): item is VisibleRoute => Boolean(item.route));
}

function visibleRouteIds(protocol: Protocol): string[] {
  return (props.routeTable?.[protocol] ?? []).map((entry) => entry.id);
}

function availableRoutes(protocol: Protocol): RouteRule[] {
  const visibleIds = visibleRouteIds(protocol);

  return props.routes[protocol].filter((route) => !visibleIds.includes(route.id));
}

function onAddRoutes(protocol: Protocol, ids: string[]): void {
  pickerOpen[protocol] = false;
  emit('addRoutes', protocol, ids);
}

const {
  clearDragState,
  clearDropTarget,
  dropPlacement,
  isDragging,
  onDragOver,
  onDragStart,
  onDrop,
} = useRouteDragSort({
  canDragRoute: () => !props.saving,
  moveRoute: (protocol, routeId, direction) => emit('moveRoute', protocol, routeId, direction),
  routeIds: (protocol) => visibleRouteIds(protocol),
});
</script>

<template>
  <section class="panel">
    <div class="panel-header">
      <div>
        <h2>{{ t('routeTable') }}</h2>
        <p v-if="selectedRouteTable" class="panel-note">
          {{ selectedRouteTable }}
          {{ activeRouteTable === selectedRouteTable ? t('isActive') : t('isInactive') }}
        </p>
        <p v-else class="panel-note">{{ t('newRouteTable') }}</p>
      </div>
      <button
        class="ghost-button compact"
        type="button"
        :disabled="!selectedRouteTable || activeRouteTable === selectedRouteTable || activating"
        @click="emit('activate')"
      >
        {{ activating ? t('activating') : t('activate') }}
      </button>
    </div>

    <div v-if="routeTable" class="route-table-layout">
      <RouteTableForm
        :deleting="deleting"
        :route-table-name="routeTableName"
        :saving="saving"
        :selected-route-table="selectedRouteTable"
        @delete="emit('delete')"
        @save="emit('save')"
        @update:route-table-name="emit('update:routeTableName', $event)"
      />

      <div class="route-table-rules">
        <section
          v-for="protocol in ['openai', 'anthropic'] as Protocol[]"
          :key="protocol"
          class="route-table-section"
        >
          <div class="route-table-section-header">
            <h3>{{ t('rulesSection', { protocol: protocolLabel(protocol) }) }}</h3>
            <button
              class="ghost-button compact"
              type="button"
              :disabled="!selectedRouteTable || saving"
              @click="pickerOpen[protocol] = !pickerOpen[protocol]"
            >
              {{ t('addRules') }}
            </button>
          </div>

          <RouteRulePicker
            v-if="pickerOpen[protocol]"
            :disabled="saving"
            :routes="availableRoutes(protocol)"
            @add="onAddRoutes(protocol, $event)"
            @cancel="pickerOpen[protocol] = false"
          />

          <div v-if="visibleRoutes(protocol).length === 0" class="empty-state">
            {{ routes[protocol].length === 0 ? t('noRules') : t('noVisibleRules') }}
          </div>
          <div v-else class="route-toggle-list">
            <RouteToggleRow
              v-for="item in visibleRoutes(protocol)"
              :key="item.route.id"
              :disabled="saving"
              :draggable="!saving"
              :dragging="isDragging(protocol, item.route.id)"
              :drop-placement="dropPlacement(protocol, item.route.id)"
              :enabled="item.entry.enabled"
              :route="item.route"
              @dragend="clearDragState"
              @dragleave="clearDropTarget"
              @dragover="onDragOver(protocol, item.route.id, $event)"
              @dragstart="onDragStart(protocol, item.route.id, $event)"
              @drop="onDrop(protocol, item.route.id, $event)"
              @move="emit('moveRoute', protocol, item.route.id, $event)"
              @remove="emit('removeRoute', protocol, item.route.id)"
              @toggle="emit('toggleRoute', protocol, item.route.id, $event)"
            />
          </div>
        </section>
      </div>
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

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.panel-header h2,
.panel-header p,
.route-table-section h3 {
  margin: 0;
}

.panel-header h2 {
  color: var(--text);
  font-size: 15px;
  line-height: 1.3;
  letter-spacing: 0;
}

.panel-note {
  display: inline-flex;
  align-items: center;
  max-width: 100%;
  min-height: 26px;
  margin-top: 8px;
  padding: 4px 8px;
  color: #536276;
  border-left: 3px solid var(--accent-border);
  border-radius: 0 var(--radius) var(--radius) 0;
  background: rgba(237, 244, 255, 0.68);
  font-size: 12px;
  line-height: 1.4;
}

.route-table-layout,
.route-table-rules,
.route-toggle-list,
.route-table-section {
  display: grid;
  align-content: start;
  gap: 10px;
}

.route-table-layout {
  gap: 16px;
}

.route-table-section {
  gap: 12px;
  padding-top: 4px;
}

.route-table-section h3 {
  color: #334155;
  font-size: 13px;
  line-height: 1.3;
}

.route-table-section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}
</style>

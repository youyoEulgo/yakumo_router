<script setup lang="ts">
import { reactive } from 'vue';
import type {
  EditorPane,
  ProviderConfig,
  ProviderTables,
  Protocol,
  RouteTables,
  RouteTableState,
} from '../types';
import { protocolLabels } from '../types';
import NewNavRow from './NewNavRow.vue';
import ProviderNavRow from './ProviderNavRow.vue';
import RouteTableNavRow from './RouteTableNavRow.vue';
import SidebarSection from './SidebarSection.vue';

const props = defineProps<{
  activePane: EditorPane;
  activeProtocol: Protocol;
  loading: boolean;
  providers: ProviderTables;
  routes: RouteTables;
  routeTables: RouteTableState;
  selectedProvider: string;
  selectedRouteTable: string;
}>();

const emit = defineEmits<{
  newProvider: [protocol: Protocol];
  selectProvider: [protocol: Protocol, name: string, provider: ProviderConfig];
  newRouteTable: [];
  selectRouteTable: [name: string];
}>();

const collapsed = reactive<Record<Protocol | 'routeTables', boolean>>({
  openai: false,
  anthropic: false,
  routeTables: false,
});

function providerEntries(protocol: Protocol): [string, ProviderConfig][] {
  return Object.entries(props.providers[protocol]).sort(([left], [right]) =>
    left.localeCompare(right),
  );
}

function providerRouteCount(protocol: Protocol, provider: string): number {
  return props.routes[protocol].filter((route) => route.provider === provider).length;
}

function routeTableEntries(): string[] {
  return Object.keys(props.routeTables.tables).sort((left, right) => left.localeCompare(right));
}

function routeTableRuleCount(name: string, protocol: Protocol): number {
  return props.routeTables.tables[name]?.[protocol].length ?? 0;
}
</script>

<template>
  <aside class="sidebar">
    <SidebarSection
      v-for="protocol in ['openai', 'anthropic'] as Protocol[]"
      :key="protocol"
      :collapsed="collapsed[protocol]"
      :title="`${protocolLabels[protocol]} Providers`"
      @toggle="collapsed[protocol] = !collapsed[protocol]"
    >
      <div v-if="loading" class="empty-state">Loading providers...</div>
      <div v-else-if="providerEntries(protocol).length === 0" class="empty-state">
        No providers configured.
      </div>
      <template v-else>
        <ProviderNavRow
          v-for="[name, provider] in providerEntries(protocol)"
          :key="`${protocol}-${name}`"
          :name="name"
          :provider="provider"
          :route-count="providerRouteCount(protocol, name)"
          :selected="
            activePane === 'provider' && activeProtocol === protocol && selectedProvider === name
          "
          @click="emit('selectProvider', protocol, name, provider)"
        />
      </template>

      <NewNavRow
        :label="`New ${protocolLabels[protocol]} provider`"
        @click="emit('newProvider', protocol)"
      />
    </SidebarSection>

    <SidebarSection
      :collapsed="collapsed.routeTables"
      title="Route Tables"
      @toggle="collapsed.routeTables = !collapsed.routeTables"
    >
      <div v-if="routeTableEntries().length === 0" class="empty-state">
        No route tables configured.
      </div>
      <template v-else>
        <RouteTableNavRow
          v-for="name in routeTableEntries()"
          :key="name"
          :active="routeTables.active === name"
          :anthropic-rule-count="routeTableRuleCount(name, 'anthropic')"
          :name="name"
          :openai-rule-count="routeTableRuleCount(name, 'openai')"
          :selected="activePane === 'route-table' && selectedRouteTable === name"
          @click="emit('selectRouteTable', name)"
        />
      </template>

      <NewNavRow label="New route table" @click="emit('newRouteTable')" />
    </SidebarSection>
  </aside>
</template>

<style scoped>
.sidebar {
  display: flex;
  flex-direction: column;
  gap: 14px;
  padding: 20px;
  border-right: 1px solid var(--border);
  background:
    linear-gradient(180deg, rgba(255, 255, 255, 0.72), rgba(243, 247, 251, 0.84)), var(--surface);
}

:deep(.nav-row) {
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

:deep(.nav-row:hover) {
  border-color: var(--accent-border);
  background: var(--surface);
  box-shadow: var(--shadow-md);
  transform: translateY(-1px);
}

:deep(.nav-row.selected) {
  border-color: var(--accent);
  background: var(--accent-soft);
  box-shadow:
    inset 4px 0 0 var(--accent),
    var(--shadow-sm);
}

:deep(.nav-row.active) {
  border-color: var(--accent-border);
  background:
    linear-gradient(180deg, rgba(237, 244, 255, 0.98), rgba(247, 250, 255, 0.96)),
    var(--accent-soft);
  box-shadow:
    inset 4px 0 0 var(--accent),
    0 0 0 1px rgba(39, 100, 216, 0.08),
    var(--shadow-sm);
}

:deep(.nav-row.active.selected) {
  border-color: var(--accent);
  box-shadow:
    inset 4px 0 0 var(--accent),
    0 0 0 1px rgba(39, 100, 216, 0.14),
    var(--shadow-md);
}

:deep(.nav-row-name) {
  min-width: 0;
  overflow: hidden;
  color: var(--text);
  font-size: 14px;
  font-weight: 700;
  line-height: 1.3;
  text-overflow: ellipsis;
  white-space: nowrap;
}

:deep(.nav-row-detail) {
  overflow: hidden;
  color: var(--text-muted);
  font-size: 12px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

:deep(.nav-row-count) {
  justify-self: start;
  min-height: 22px;
  padding: 3px 8px;
  color: #4b5d73;
  border: 1px solid #e2e8f0;
  border-radius: 999px;
  background: var(--surface-subtle);
  font-size: 12px;
  line-height: 1.2;
}

:deep(.nav-row-meta) {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  align-items: center;
}

:deep(.active-badge) {
  min-height: 22px;
  padding: 3px 8px;
  color: #ffffff;
  border: 1px solid var(--accent);
  border-radius: 999px;
  background: var(--accent);
  font-size: 12px;
  font-weight: 600;
  line-height: 1.2;
}

:deep(.new-row) {
  place-items: center;
  min-height: 56px;
  color: var(--text-muted);
  border-style: dashed;
  background: rgba(255, 255, 255, 0.6);
  box-shadow: none;
}

:deep(.new-row:hover) {
  color: var(--accent);
  border-color: var(--accent-border);
  background: var(--accent-soft);
}

:deep(.new-row-plus) {
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
  .sidebar {
    border-right: 0;
    border-bottom: 1px solid var(--border);
  }
}

@media (max-width: 760px) {
  .sidebar {
    padding: 18px;
  }
}
</style>

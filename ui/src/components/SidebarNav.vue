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

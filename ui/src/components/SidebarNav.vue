<script setup lang="ts">
import type { EditorPane, ProviderConfig, ProviderTables, Protocol, RouteTables, RouteTableState } from '../types';
import { protocolLabels } from '../types';

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

function providerEntries(protocol: Protocol): [string, ProviderConfig][] {
  return Object.entries(props.providers[protocol]).sort(([left], [right]) => left.localeCompare(right));
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
    <template v-for="protocol in (['openai', 'anthropic'] as Protocol[])" :key="protocol">
      <div class="list-header">
        <h2>{{ protocolLabels[protocol] }} Providers</h2>
        <button type="button" class="primary-button compact" @click="emit('newProvider', protocol)">
          New
        </button>
      </div>

      <div v-if="loading" class="empty-state">Loading providers...</div>
      <div v-else-if="providerEntries(protocol).length === 0" class="empty-state">
        No providers configured.
      </div>
      <template v-else>
        <button
          v-for="[name, provider] in providerEntries(protocol)"
          :key="`${protocol}-${name}`"
          type="button"
          class="provider-row"
          :class="{
            selected:
              activePane === 'provider' &&
              activeProtocol === protocol &&
              selectedProvider === name,
          }"
          @click="emit('selectProvider', protocol, name, provider)"
        >
          <span class="provider-name">{{ name }}</span>
          <span class="provider-url">{{ provider.base_url }}</span>
          <span class="provider-count">{{ providerRouteCount(protocol, name) }} rules</span>
        </button>
      </template>
    </template>

    <div class="list-header">
      <h2>Route Tables</h2>
      <button type="button" class="primary-button compact" @click="emit('newRouteTable')">
        New
      </button>
    </div>

    <div v-if="routeTableEntries().length === 0" class="empty-state">
      No route tables configured.
    </div>
    <template v-else>
      <button
        v-for="name in routeTableEntries()"
        :key="name"
        type="button"
        class="provider-row"
        :class="{ selected: activePane === 'route-table' && selectedRouteTable === name }"
        @click="emit('selectRouteTable', name)"
      >
        <span class="provider-name">{{ name }}</span>
        <span class="provider-url">
          {{ routeTables.active === name ? 'Active route table' : 'Inactive' }}
        </span>
        <span class="provider-count">
          {{ routeTableRuleCount(name, 'openai') }} O /
          {{ routeTableRuleCount(name, 'anthropic') }} A rules
        </span>
      </button>
    </template>
  </aside>
</template>

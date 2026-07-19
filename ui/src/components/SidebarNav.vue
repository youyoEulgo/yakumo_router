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
    <template v-for="protocol in ['openai', 'anthropic'] as Protocol[]" :key="protocol">
      <button
        type="button"
        class="list-header collapsible-header"
        :aria-expanded="!collapsed[protocol]"
        @click="collapsed[protocol] = !collapsed[protocol]"
      >
        <h2>{{ protocolLabels[protocol] }} Providers</h2>
        <span
          class="collapse-button"
          :class="{ collapsed: collapsed[protocol] }"
          aria-hidden="true"
        >
          ▾
        </span>
      </button>

      <template v-if="!collapsed[protocol]">
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

        <button
          type="button"
          class="provider-row new-row"
          :aria-label="`New ${protocolLabels[protocol]} provider`"
          @click="emit('newProvider', protocol)"
        >
          <span class="new-row-plus" aria-hidden="true">+</span>
        </button>
      </template>
    </template>

    <button
      type="button"
      class="list-header collapsible-header"
      :aria-expanded="!collapsed.routeTables"
      @click="collapsed.routeTables = !collapsed.routeTables"
    >
      <h2>Route Tables</h2>
      <span
        class="collapse-button"
        :class="{ collapsed: collapsed.routeTables }"
        aria-hidden="true"
      >
        ▾
      </span>
    </button>

    <template v-if="!collapsed.routeTables">
      <div v-if="routeTableEntries().length === 0" class="empty-state">
        No route tables configured.
      </div>
      <template v-else>
        <button
          v-for="name in routeTableEntries()"
          :key="name"
          type="button"
          class="provider-row"
          :class="{
            selected: activePane === 'route-table' && selectedRouteTable === name,
            active: routeTables.active === name,
          }"
          @click="emit('selectRouteTable', name)"
        >
          <span class="provider-name">{{ name }}</span>
          <span class="provider-url">
            {{ routeTables.active === name ? 'Active route table' : 'Inactive' }}
          </span>
          <span class="provider-row-meta">
            <span v-if="routeTables.active === name" class="active-badge">Active</span>
            <span class="provider-count">
              {{ routeTableRuleCount(name, 'openai') }} OpenAI /
              {{ routeTableRuleCount(name, 'anthropic') }} Anthropic rules
            </span>
          </span>
        </button>
      </template>

      <button
        type="button"
        class="provider-row new-row"
        aria-label="New route table"
        @click="emit('newRouteTable')"
      >
        <span class="new-row-plus" aria-hidden="true">+</span>
      </button>
    </template>
  </aside>
</template>

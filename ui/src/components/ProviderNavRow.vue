<script setup lang="ts">
import './sidebar-nav-row.css';
import type { ProviderConfig } from '../types';

defineProps<{
  name: string;
  provider: ProviderConfig;
  routeCount: number;
  selected: boolean;
}>();
</script>

<template>
  <button type="button" class="nav-row" :class="{ selected }">
    <span class="nav-row-name">{{ name }}</span>
    <span class="nav-row-detail">{{ provider.base_url }}</span>
    <span class="nav-row-count">{{ routeCount }} rules</span>
  </button>
</template>

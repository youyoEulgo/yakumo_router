<script setup lang="ts">
import './sidebar-nav-row.css';

defineProps<{
  active: boolean;
  activeLabel: string;
  anthropicRuleCount: number;
  anthropicRulesLabel: string;
  inactiveLabel: string;
  name: string;
  openaiRuleCount: number;
  openaiRulesLabel: string;
  selected: boolean;
}>();
</script>

<template>
  <button type="button" class="nav-row" :class="{ active, selected }">
    <span class="nav-row-name">{{ name }}</span>
    <span class="nav-row-detail">
      {{ active ? activeLabel : inactiveLabel }}
    </span>
    <span class="nav-row-meta">
      <span v-if="active" class="active-badge">{{ activeLabel }}</span>
      <span class="nav-row-count">
        {{ openaiRuleCount }} {{ openaiRulesLabel }} / {{ anthropicRuleCount }}
        {{ anthropicRulesLabel }}
      </span>
    </span>
  </button>
</template>

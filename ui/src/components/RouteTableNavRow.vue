<script setup lang="ts">
defineProps<{
  active: boolean;
  anthropicRuleCount: number;
  name: string;
  openaiRuleCount: number;
  selected: boolean;
}>();
</script>

<template>
  <button type="button" class="nav-row" :class="{ active, selected }">
    <span class="nav-row-name">{{ name }}</span>
    <span class="nav-row-detail">
      {{ active ? 'Active route table' : 'Inactive' }}
    </span>
    <span class="nav-row-meta">
      <span v-if="active" class="active-badge">Active</span>
      <span class="nav-row-count">
        {{ openaiRuleCount }} OpenAI / {{ anthropicRuleCount }} Anthropic rules
      </span>
    </span>
  </button>
</template>

<script setup lang="ts">
import './sidebar-nav-row.css';

defineProps<{
  label: string;
}>();
</script>

<template>
  <button type="button" class="nav-row new-row" :aria-label="label">
    <span class="new-row-plus" aria-hidden="true">+</span>
  </button>
</template>

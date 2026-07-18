<script setup lang="ts">
defineProps<{
  error: string;
  status: string;
}>();
</script>

<template>
  <div v-if="status || error" class="toast-stack" aria-live="polite">
    <p v-if="status" class="toast status">{{ status }}</p>
    <p v-if="error" class="toast error">{{ error }}</p>
  </div>
</template>

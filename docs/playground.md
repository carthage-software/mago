---
title: Playground
layout: page
sidebar: false
---

<script setup>
import MagoPlayground from './.vitepress/theme/components/playground/MagoPlayground.vue';
</script>

<MagoPlayground />

<style>
.VPSidebar {
  display: none !important;
}

.VPDoc .content {
  max-width: 100% !important;
  padding: 0 24px !important;
}

@media (max-width: 768px) {
  .VPDoc .content {
    padding: 0 16px !important;
  }
}
</style>

<script setup lang="ts">
import Receive from "./page/Receive.vue";
import Send from "./page/Send.vue";
import Settings from "./page/Settings.vue";
</script>

<template>
  <main class="container"><keep-alive>
    <n-tabs
      default-value="recv"
      justify-content="space-evenly"
      type="line"
      placement="bottom"
      size="large"
      animated
      :destroy-on-hide="false"
    >

        <n-tab-pane
          name="recv"
          tab="接收"
          class="my-pane"
          style="padding: 10px"
        >
          <component :is="Receive"></component>
        </n-tab-pane>
        <n-tab-pane
          name="send"
          tab="发送"
          class="my-pane"
          style="padding: 10px"
        >
          <component :is="Send"></component>
        </n-tab-pane>
        <n-tab-pane
          name="setting"
          tab="设置"
          class="my-pane"
          style="padding: 10px"
        >
          <component :is="Settings"></component>
        </n-tab-pane>

    </n-tabs>      </keep-alive>
  </main>
</template>

<style scoped>
.container {
  -webkit-user-select: none;
  -moz-user-select: none;
  -ms-user-select: none;
  user-select: none;
}
.my-pane {
  height: calc(100vh - 73px);
  width: calc(100vw - 20px);
  overflow: auto;
}
</style>

<script setup lang="ts">
import { onMounted, ref } from "vue";
import { listen } from "@tauri-apps/api/event";
import { DeviceMessage } from "../model";
import { RefreshOutline } from "@vicons/ionicons5";
import { invoke } from "@tauri-apps/api/core";

let devices = ref<Array<[string, DeviceMessage]>>([]);

listen<[string, DeviceMessage]>("device-connect", (event) => {
  devices.value.push(event.payload);
});

const refresh = async () => {
  devices.value = [];
  await invoke("refresh");
};

onMounted(() => {
    refresh();
})
</script>

<template>
  <div>
    <n-h1
      >附近的设备
      <n-button strong secondary circle type="primary" @click="refresh">
        <template #icon>
          <n-icon><RefreshOutline /></n-icon> </template
      ></n-button>
    </n-h1>
    <n-list hoverable clickable>
      <n-list-item v-for="(device, index) in devices" :key="index">
        <n-thing :title="device[1].alias" content-style="margin-top: 10px;">
          <template #description>
            <n-space>
              <n-tag :bordered="false" type="info" size="small" round>
                {{ device[1].deviceType }}
              </n-tag>
              <n-tag :bordered="false" type="info" size="small" round v-if="device[1].deviceModel !=='' ">
                {{ device[1].deviceModel }}
              </n-tag>
              <n-tag :bordered="false" type="success" size="small" round>
                {{ device[1].protocol }}
              </n-tag>
            </n-space>
          </template>
          IP: {{ device[0] }}<br />
        </n-thing>
      </n-list-item>
    </n-list>
  </div>
</template>

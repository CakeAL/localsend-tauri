<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { ref } from "vue";
import { DeviceMessage } from "../model";

const deviceInfo = ref<[DeviceMessage | null, Array<string>]>([null, [""]]);

const getDeviceInfo = async () => {
  let res = await invoke<string>("get_device_info");
  deviceInfo.value = JSON.parse(res);
  // console.log(deviceInfo.value);
};

getDeviceInfo();
</script>

<template>
  <div>
    <n-h1>Receive</n-h1>
    <div v-if="deviceInfo[0]">
      <n-p><strong>设备名:</strong> {{ deviceInfo[0].alias }}</n-p>
      <n-p><strong>端口:</strong> {{ deviceInfo[0].port }}</n-p>
      <n-p><strong>设备类型:</strong> {{ deviceInfo[0].deviceType }}</n-p>
      <n-p><strong>IP:</strong></n-p>
      <n-p v-for="(item, index) in deviceInfo[1]" :key="index">{{ item }}</n-p>
    </div>
  </div>
</template>

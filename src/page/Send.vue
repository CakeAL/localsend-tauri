<script setup lang="ts">
import { onMounted, ref } from "vue";
import { listen } from "@tauri-apps/api/event";
import { DeviceMessage, FileInfo } from "../model";
import { RefreshOutline } from "@vicons/ionicons5";
import { invoke } from "@tauri-apps/api/core";
import { showFileSize } from "../util";

let devices = ref<Array<[string, DeviceMessage]>>([]);
const fileInfos = ref<Array<FileInfo>>([]);
const idPath = ref<Record<string, string>>();

listen<[string, DeviceMessage]>("device-connect", (event) => {
  devices.value.push(event.payload);
});

const refresh = async () => {
  await invoke("refresh");
};

onMounted(() => {
  refresh();
});

const openFilePicker = async () => {
  let res = await invoke<string>("open_file_picker");
  const parsed = JSON.parse(res);
  idPath.value = parsed[0];
  fileInfos.value = parsed[1];
  console.log(fileInfos.value);
};

const prepareUploadFiles = async (addr: string, port: number) => {  
  await invoke("prepare_upload_files", {
    idPath: idPath.value,
    fileInfos: fileInfos.value,
    addr: addr,
    port: port,
  }).catch((err) => alert(err));
};
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
      <n-list-item
        v-for="(device, index) in devices"
        :key="index"
        @click="prepareUploadFiles(device[0], device[1].port)"
      >
        <n-thing :title="device[1].alias" content-style="margin-top: 10px;">
          <template #description>
            <n-space>
              <n-tag :bordered="false" type="info" size="small" round>
                {{ device[1].deviceType }}
              </n-tag>
              <n-tag
                :bordered="false"
                type="info"
                size="small"
                round
                v-if="device[1].deviceModel !== ''"
              >
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
    <hr />
    <n-button type="success" @click="openFilePicker"> 选择文件 </n-button>
    <n-list hoverable clickable>
      <n-list-item v-for="(file, index) in fileInfos" :key="index">
        <n-thing :title="file.fileName" content-style="margin-top: 10px;">
          <template #description>
            <n-space>
              <n-tag :bordered="false" type="info" size="small" round>
                {{ file.fileType }}
              </n-tag>
              <n-tag :bordered="false" type="success" size="small" round>
                {{ showFileSize(file.size) }}
              </n-tag>
            </n-space>
          </template>
        </n-thing>
      </n-list-item>
    </n-list>
  </div>
</template>

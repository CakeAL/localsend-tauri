<script setup lang="ts">
import { ref } from "vue";
import Receive from "./page/Receive.vue";
import Send from "./page/Send.vue";
import Settings from "./page/Settings.vue";
import { emit, listen } from "@tauri-apps/api/event";
import { FileRequest } from "./model";

const fileReq = ref<FileRequest>();
const active = ref(false);

listen<FileRequest>("file-prepare-upload", (event) => {
  // console.log(event.payload);
  fileReq.value = event.payload;
  active.value = true;
});

function showFileSize(size: number): string {
  if (size < 1024) {
    return `${size} Bytes`;
  } else if (size < Math.pow(1024, 2)) {
    return `${(size / 1024).toFixed(2)} KB`;
  } else if (size < Math.pow(1024, 3)) {
    return `${(size / Math.pow(1024, 2)).toFixed(2)} MB`;
  } else if (size < Math.pow(1024, 4)) {
    return `${(size / Math.pow(1024, 3)).toFixed(2)} GB`;
  } else {
    return `${(size / Math.pow(1024, 4)).toFixed(2)} TB`;
  }
}

const agreed = async () => {
  // 全部同意😓
  let agreed_set: Array<string> = [];
  Object.values(fileReq.value!.files).forEach((file) => {
    agreed_set.push(file.id);
  });
  console.log(agreed_set);
  await emit("agreed-set", agreed_set);
};
</script>

<template>
  <main class="container">
    <n-tabs
      default-value="recv"
      justify-content="space-evenly"
      type="line"
      placement="bottom"
      size="large"
      animated
      :destroy-on-hide="false"
    >
      <n-tab-pane name="recv" tab="接收" class="my-pane" style="padding: 10px">
        <component :is="Receive"></component>
      </n-tab-pane>
      <n-tab-pane name="send" tab="发送" class="my-pane" style="padding: 10px">
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
    </n-tabs>
    <n-drawer v-model:show="active" height="90vh" placement="bottom">
      <n-drawer-content title="文件传入请求">
        <n-card :title="fileReq?.info.alias" hoverable>
          <n-list hoverable clickable>
            <n-list-item
              v-for="[key, file] in Object.entries(fileReq?.files || {})"
              :key="key"
            >
              <n-thing :title="file.fileName" content-style="margin-top: 10px;">
                <template #description>
                  <n-space>
                    <n-tag :bordered="false" type="info" size="small" round>
                      {{ file.fileType }}
                    </n-tag>
                  </n-space>
                </template>
                大小:
                {{ showFileSize(file.size) }}
              </n-thing>
            </n-list-item>
          </n-list>
        </n-card>
        <template #footer>
          <n-button type="primary" size="large" @click="agreed">同意</n-button>
        </template>
      </n-drawer-content>
    </n-drawer>
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

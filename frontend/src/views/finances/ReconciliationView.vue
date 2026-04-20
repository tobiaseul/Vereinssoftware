<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { ElMessage } from 'element-plus'

const route = useRoute()

const accountId = computed(() => route.params.id as string)

function handleFileUpload(file: File) {
  // TODO: Implement file upload and reconciliation
  ElMessage.info('File upload coming soon')
}
</script>

<template>
  <div class="reconciliation">
    <h1>Bank Reconciliation</h1>

    <el-card>
      <div class="upload-section">
        <p>Upload a bank statement file (CSV/OFX) to reconcile account {{ accountId }}</p>
        <el-upload
          drag
          action="#"
          :auto-upload="false"
          @change="handleFileUpload"
        >
          <el-icon class="el-icon--upload">
            <svg viewBox="0 0 1024 1024"><path fill="currentColor" d="M512 64C265.056 64 64 265.056 64 512s201.056 448 448 448 448-201.056 448-448S758.944 64 512 64Zm0 832c-211.744 0-384-172.256-384-384s172.256-384 384-384 384 172.256 384 384-172.256 384-384 384Z"/></svg>
          </el-icon>
          <div class="el-upload__text">
            Drop file here or <em>click to upload</em>
          </div>
        </el-upload>
      </div>
    </el-card>
  </div>
</template>

<style scoped>
.reconciliation {
  padding: 20px;
}

.upload-section {
  padding: 20px;
}
</style>

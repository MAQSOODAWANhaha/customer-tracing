<template>
  <div class="not-found-container">
    <n-result
      status="404"
      title="页面不存在"
      description="您访问的页面可能已被删除或不存在"
    >
      <template #footer>
        <n-space>
          <n-button @click="goBack" secondary>
            返回上一页
          </n-button>
          <n-button @click="goHome" type="primary">
            回到首页
          </n-button>
        </n-space>
      </template>
    </n-result>
  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router'

const router = useRouter()

const goBack = () => {
  router.back()
}

const goHome = () => {
  router.push('/customers')
}
</script>

<style scoped>
.not-found-container {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #f5f7fa;
}
</style>
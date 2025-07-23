<template>
  <div class="login-container">
    <div class="login-card">
      <n-card title="客户追踪系统" size="large">
        <template #header-extra>
          <n-icon :component="PersonCircleOutline" size="24" />
        </template>
        
        <n-form
          ref="loginFormRef"
          :model="loginForm"
          :rules="loginFormRules"
          size="large"
          :show-require-mark="false"
        >
          <n-form-item path="username">
            <n-input
              v-model:value="loginForm.username"
              placeholder="请输入用户名"
              clearable
              @keydown.enter="handleLogin"
            >
              <template #prefix>
                <n-icon :component="PersonOutline" />
              </template>
            </n-input>
          </n-form-item>
          
          <n-form-item path="password">
            <n-input
              v-model:value="loginForm.password"
              type="password"
              placeholder="请输入密码"
              show-password-on="click"
              @keydown.enter="handleLogin"
            >
              <template #prefix>
                <n-icon :component="LockClosedOutline" />
              </template>
            </n-input>
          </n-form-item>
          
          <n-form-item>
            <n-checkbox v-model:checked="rememberMe">
              记住登录状态
            </n-checkbox>
          </n-form-item>
        </n-form>
        
        <template #action>
          <n-space vertical size="large" style="width: 100%">
            <n-button
              type="primary"
              size="large"
              block
              :loading="authStore.loading"
              @click="handleLogin"
            >
              登录
            </n-button>
            
            <n-text depth="3" style="text-align: center; display: block">
              客户追踪管理系统 v1.0
            </n-text>
          </n-space>
        </template>
      </n-card>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useMessage, type FormInst, type FormRules } from 'naive-ui'
import { PersonCircleOutline, PersonOutline, LockClosedOutline } from '@vicons/ionicons5'
import { useAuthStore } from '@/stores/auth'
import type { LoginRequest } from '@/types'

const router = useRouter()
const route = useRoute()
const message = useMessage()
const authStore = useAuthStore()

const loginFormRef = ref<FormInst | null>(null)
const rememberMe = ref(true)

const loginForm = reactive<LoginRequest>({
  username: '',
  password: ''
})

const loginFormRules: FormRules = {
  username: [
    { required: true, message: '请输入用户名', trigger: ['input', 'blur'] },
    { min: 3, max: 50, message: '用户名长度为 3-50 个字符', trigger: ['input', 'blur'] }
  ],
  password: [
    { required: true, message: '请输入密码', trigger: ['input', 'blur'] },
    { min: 6, message: '密码长度不能少于 6 个字符', trigger: ['input', 'blur'] }
  ]
}

const handleLogin = async () => {
  if (!loginFormRef.value) return
  
  try {
    await loginFormRef.value.validate()
    
    const result = await authStore.login(loginForm)
    
    if (result.success) {
      message.success(`欢迎回来，${result.user?.name}！`)
      
      // 跳转到目标页面或首页
      const redirect = route.query.redirect as string || '/customers'
      router.push(redirect)
    } else {
      message.error(result.message || '登录失败')
    }
  } catch (error) {
    // 表单验证失败
  }
}

// 页面加载时检查是否已登录
onMounted(() => {
  if (authStore.isAuthenticated) {
    router.push('/customers')
  }
})
</script>

<style scoped>
.login-container {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  padding: 20px;
}

.login-card {
  width: 100%;
  max-width: 400px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
  border-radius: 16px;
  overflow: hidden;
  backdrop-filter: blur(8px);
}

.login-card :deep(.n-card) {
  background: rgba(255, 255, 255, 0.95);
}

.login-card :deep(.n-card-header) {
  text-align: center;
  font-weight: 600;
  font-size: 20px;
  color: #333;
}

@media (max-width: 480px) {
  .login-container {
    padding: 16px;
  }
  
  .login-card {
    max-width: none;
  }
}
</style>
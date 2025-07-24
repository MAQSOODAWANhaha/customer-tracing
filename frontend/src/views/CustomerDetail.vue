<template>
  <div class="customer-detail-container">
    <!-- 页面头部 -->
    <n-card class="header-card" :bordered="false">
      <div class="header-content">
        <div class="header-left">
          <n-button 
            text 
            @click="handleBack"
            class="back-btn"
          >
            <template #icon>
              <n-icon :component="ArrowBackOutline" />
            </template>
            返回
          </n-button>
          
          <div class="customer-title" v-if="customer">
            <h1>{{ customer.name }}</h1>
            <n-space align="center" size="small">
              <n-tag 
                :type="getStatusTagType(customer.next_action)" 
                round
              >
                {{ customer.next_action }}
              </n-tag>
              <n-text depth="3" style="font-size: 13px;">
                创建于 {{ formatDate(customer.created_at) }}
              </n-text>
            </n-space>
          </div>
        </div>
        
        <n-space v-if="customer">
          <n-button 
            @click="showEditModal = true"
            secondary
          >
            <template #icon>
              <n-icon :component="CreateOutline" />
            </template>
            编辑
          </n-button>
          <n-button 
            type="primary"
            @click="showTrackModal = true"
          >
            <template #icon>
              <n-icon :component="AddCircleOutline" />
            </template>
            添加跟进
          </n-button>
        </n-space>
      </div>
    </n-card>

    <!-- 主要内容 -->
    <n-spin :show="loading">
      <div v-if="customer" class="detail-content">
        <n-grid :cols="24" :x-gap="16" :y-gap="16">
          <!-- 客户信息卡片 -->
          <n-gi :span="24" :lg="8">
            <n-card title="客户信息" class="info-card">
              <div class="customer-info">
                <div class="info-section">
                  <div class="info-item" v-if="customer.phone">
                    <div class="info-label">
                      <n-icon :component="CallOutline" />
                      <span>手机号码</span>
                    </div>
                    <div class="info-value">
                      <n-text>{{ customer.phone }}</n-text>
                      <n-button 
                        text 
                        size="tiny" 
                        @click="handleCall"
                        class="action-link"
                      >
                        拨打
                      </n-button>
                    </div>
                  </div>

                  <div class="info-item">
                    <div class="info-label">
                      <n-icon :component="StarOutline" />
                      <span>客户评级</span>
                    </div>
                    <div class="info-value">
                      <n-rate :value="customer.rate" readonly size="small" />
                      <n-text depth="3" style="font-size: 12px; margin-left: 8px;">
                        {{ customer.rate }}/5
                      </n-text>
                    </div>
                  </div>

                  <div class="info-item" v-if="customer.address">
                    <div class="info-label">
                      <n-icon :component="LocationOutline" />
                      <span>联系地址</span>
                    </div>
                    <div class="info-value">
                      <n-text>{{ customer.address }}</n-text>
                    </div>
                  </div>
                </div>

                <n-divider v-if="customer.notes" />

                <div v-if="customer.notes" class="notes-section">
                  <div class="info-label">
                    <n-icon :component="DocumentTextOutline" />
                    <span>备注信息</span>
                  </div>
                  <div class="notes-content">
                    <n-text>{{ customer.notes }}</n-text>
                  </div>
                </div>
              </div>
            </n-card>

            <!-- 统计信息 -->
            <n-card title="跟进统计" class="stats-card">
              <div class="stats-grid">
                <div class="stat-item">
                  <div class="stat-value">{{ customer.track_count || 0 }}</div>
                  <div class="stat-label">跟进次数</div>
                </div>
                <div class="stat-item">
                  <div class="stat-value">
                    {{ customer.last_track_at ? daysSinceLastTrack : '-' }}
                  </div>
                  <div class="stat-label">距上次跟进</div>
                </div>
              </div>
            </n-card>
          </n-gi>

          <!-- 跟进记录时间线 -->
          <n-gi :span="24" :lg="16">
            <tracking-timeline 
              :customer-id="customerId" 
              @refresh="handleRefreshCustomer"
              @add-track="showTrackModal = true"
            />
          </n-gi>
        </n-grid>
      </div>

      <!-- 空状态 -->
      <div v-else-if="!loading" class="empty-state">
        <n-result
          status="404"
          title="客户不存在"
          description="该客户可能已被删除或您没有访问权限"
        >
          <template #footer>
            <n-button @click="handleBack" type="primary">
              返回客户列表
            </n-button>
          </template>
        </n-result>
      </div>
    </n-spin>

    <!-- 编辑客户弹窗 -->
    <customer-form-modal
      v-model:show="showEditModal"
      :customer="customer"
      @success="handleEditSuccess"
    />

    <!-- 添加跟进弹窗 -->
    <track-form-modal
      v-model:show="showTrackModal"
      :customer="customer"
      @success="handleTrackSuccess"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useMessage } from 'naive-ui'
import {
  ArrowBackOutline,
  CreateOutline,
  AddCircleOutline,
  CallOutline,
  StarOutline,
  LocationOutline,
  DocumentTextOutline
} from '@vicons/ionicons5'
import { useCustomerStore } from '@/stores/customer'
import CustomerFormModal from '@/components/CustomerFormModal.vue'
import TrackingTimeline from '@/components/TrackingTimeline.vue'
import TrackFormModal from '@/components/TrackFormModal.vue'
import type { NextAction } from '@/types'

const route = useRoute()
const router = useRouter()
const message = useMessage()
const customerStore = useCustomerStore()

const loading = ref(false)
const showEditModal = ref(false)
const showTrackModal = ref(false)

// 计算属性
const customerId = computed(() => Number(route.params.id))
const customer = computed(() => customerStore.currentCustomer)

const daysSinceLastTrack = computed(() => {
  if (!customer.value?.last_track_at) return '-'
  const lastTrack = new Date(customer.value.last_track_at)
  const now = new Date()
  const diffTime = now.getTime() - lastTrack.getTime()
  const diffDays = Math.ceil(diffTime / (1000 * 60 * 60 * 24))
  
  if (diffDays === 0) return '今天'
  if (diffDays === 1) return '1天'
  return `${diffDays}天`
})

// 获取状态标签类型
const getStatusTagType = (status: NextAction) => {
  switch (status) {
    case '继续跟进':
      return 'success'
    case '结束跟进':
      return 'default'
    default:
      return 'info'
  }
}

// 加载客户详情
const loadCustomer = async () => {
  try {
    loading.value = true
    await customerStore.fetchCustomer(customerId.value)
  } catch (error: any) {
    message.error(error.message || '加载客户详情失败')
    router.push('/customers')
  } finally {
    loading.value = false
  }
}

// 格式化日期
const formatDate = (dateString: string) => {
  const date = new Date(dateString)
  return date.toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: 'long',
    day: 'numeric'
  })
}

// 处理操作
const handleBack = () => {
  router.push('/customers')
}

const handleCall = () => {
  if (customer.value?.phone) {
    window.open(`tel:${customer.value.phone}`)
  }
}


const handleEditSuccess = () => {
  showEditModal.value = false
  loadCustomer()
  message.success('客户信息更新成功')
}

const handleTrackSuccess = () => {
  showTrackModal.value = false
  loadCustomer()
  message.success('跟进记录添加成功')
}

const handleRefreshCustomer = () => {
  loadCustomer()
}

// 监听路由参数变化
watch(
  () => route.params.id,
  (newId) => {
    if (newId) {
      loadCustomer()
    }
  }
)

// 监听哈希变化，自动滚动到跟进记录
watch(
  () => route.hash,
  (hash) => {
    if (hash === '#track') {
      showTrackModal.value = true
    }
  },
  { immediate: true }
)

// 页面加载
onMounted(() => {
  loadCustomer()
})
</script>

<style scoped>
.customer-detail-container {
  min-height: 100vh;
  background: #f5f7fa;
  padding: 20px;
}

.header-card {
  margin-bottom: 16px;
}

.header-content {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 16px;
}

.header-left {
  display: flex;
  align-items: flex-start;
  gap: 16px;
  flex: 1;
  min-width: 0;
}

.back-btn {
  flex-shrink: 0;
  margin-top: 4px;
}

.customer-title h1 {
  margin: 0 0 8px 0;
  font-size: 24px;
  font-weight: 600;
  color: #333;
  word-break: break-word;
}

.detail-content {
  min-height: 400px;
}

.info-card,
.stats-card {
  margin-bottom: 16px;
}

.customer-info {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.info-section {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.info-item {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 8px;
  padding: 8px 0;
  border-bottom: 1px solid #f0f0f0;
}

.info-item:last-child {
  border-bottom: none;
}

.info-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: #666;
  min-width: 80px;
  flex-shrink: 0;
}

.info-label .n-icon {
  font-size: 14px;
}

.info-value {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  min-width: 0;
  justify-content: space-between;
}

.info-value .n-text {
  word-break: break-all;
  line-height: 1.4;
}

.action-link {
  flex-shrink: 0;
  font-size: 11px;
}

.notes-section .info-label {
  margin-bottom: 8px;
}

.notes-content {
  padding: 12px;
  background: #f8f9fa;
  border-radius: 6px;
  border-left: 3px solid #1890ff;
}

.notes-content .n-text {
  line-height: 1.5;
  word-break: break-word;
}

.stats-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

.stat-item {
  text-align: center;
  padding: 16px;
  background: #f8f9fa;
  border-radius: 8px;
}

.stat-value {
  font-size: 24px;
  font-weight: 600;
  color: #1890ff;
  margin-bottom: 4px;
}

.stat-label {
  font-size: 12px;
  color: #666;
}

.empty-state {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 400px;
}

/* 移动端适配 */
@media (max-width: 768px) {
  .customer-detail-container {
    padding: 16px;
  }
  
  .header-content {
    flex-direction: column;
    align-items: flex-start;
  }
  
  .header-left {
    width: 100%;
  }
  
  .customer-title h1 {
    font-size: 20px;
  }
  
  .stats-grid {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 480px) {
  .customer-detail-container {
    padding: 12px;
  }
  
  .info-item {
    flex-direction: column;
    align-items: flex-start;
    gap: 4px;
  }
  
  .info-value {
    justify-content: flex-start;
  }
  
  .stat-value {
    font-size: 20px;
  }
}
</style>
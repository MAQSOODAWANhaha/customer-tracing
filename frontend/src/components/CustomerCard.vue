<template>
  <n-card 
    class="customer-card" 
    hoverable 
    @click="$emit('view', customer)"
  >
    <template #header>
      <div class="card-header">
        <div class="customer-info">
          <n-text class="customer-name">{{ customer.name }}</n-text>
          <div class="status-tags">
            <n-tag 
              size="small" 
              :type="getGroupTagType(customer.customer_group)"
              round
              class="group-tag"
            >
              {{ customer.customer_group }}
            </n-tag>
            <n-tag 
              :type="getStatusTagType(customer.next_action)" 
              size="small"
              round
              class="status-tag"
            >
              {{ customer.next_action }}
            </n-tag>
          </div>
        </div>
        <n-dropdown
          trigger="click"
          :options="dropdownOptions"
          @select="handleMenuSelect"
          @click.stop
        >
          <n-button 
            text 
            circle 
            size="small"
            @click.stop
          >
            <template #icon>
              <n-icon :component="EllipsisVerticalOutline" />
            </template>
          </n-button>
        </n-dropdown>
      </div>
    </template>

    <div class="card-content">
      <!-- 联系信息 -->
      <div class="contact-info">
        <div class="info-item" v-if="customer.phone">
          <n-icon :component="CallOutline" size="14" />
          <n-text depth="2">{{ customer.phone }}</n-text>
          <n-button 
            text 
            size="tiny" 
            @click.stop="handleCall"
            class="action-btn"
          >
            拨打
          </n-button>
        </div>
        
        <div class="info-item">
          <n-icon :component="StarOutline" size="14" />
          <n-rate :value="customer.rate" readonly size="small" />
          <n-text depth="3" style="font-size: 12px; margin-left: 4px;">
            {{ customer.rate }}/5
          </n-text>
        </div>
        
        
        <div class="info-item" v-if="customer.address">
          <n-icon :component="LocationOutline" size="14" />
          <n-text depth="2" class="address-text">{{ customer.address }}</n-text>
        </div>
      </div>

      <!-- 备注信息 -->
      <div v-if="customer.notes" class="notes-section">
        <n-text depth="3" class="notes-text">
          {{ truncateText(customer.notes, 60) }}
        </n-text>
      </div>

      <!-- 跟进统计 -->
      <div class="stats-section">
        <div class="stat-item">
          <n-text depth="3" style="font-size: 12px;">跟进次数</n-text>
          <n-text strong>{{ customer.track_count || 0 }}</n-text>
        </div>
        <div class="stat-item">
          <n-text depth="3" style="font-size: 12px;">最后跟进</n-text>
          <n-text strong style="font-size: 12px;">
            {{ customer.latest_track_time ? formatDate(customer.latest_track_time) : '无' }}
          </n-text>
        </div>
      </div>

      <!-- 最后跟进状态 -->
      <div v-if="customer.latest_next_action" class="last-track-section">
        <div class="last-track-info">
          <n-text depth="3" style="font-size: 12px;">最后状态</n-text>
          <n-tag 
            :type="getStatusTagType(customer.latest_next_action)" 
            size="small"
            round
          >
            {{ customer.latest_next_action }}
          </n-tag>
        </div>
        <div v-if="customer.latest_content" class="last-track-content">
          <n-text depth="3" style="font-size: 11px;">
            {{ truncateText(customer.latest_content, 40) }}
          </n-text>
        </div>
      </div>

      <!-- 创建时间 -->
      <div class="footer-info">
        <n-text depth="3" style="font-size: 11px;">
          创建于 {{ formatDate(customer.created_at) }}
        </n-text>
      </div>
    </div>

    <!-- 快捷操作按钮 -->
    <template #action>
      <n-space size="small">
        <n-button 
          size="small" 
          @click.stop="$emit('track', customer)"
          type="primary"
          ghost
        >
          <template #icon>
            <n-icon :component="AddCircleOutline" />
          </template>
          跟进
        </n-button>
        <n-button 
          size="small" 
          @click.stop="$emit('edit', customer)"
        >
          <template #icon>
            <n-icon :component="CreateOutline" />
          </template>
          编辑
        </n-button>
      </n-space>
    </template>
  </n-card>
</template>

<script setup lang="ts">
import { computed, h } from 'vue'
import { 
  EllipsisVerticalOutline,
  CallOutline,
  StarOutline,
  LocationOutline,
  AddCircleOutline,
  CreateOutline,
  EyeOutline,
  TrashOutline
} from '@vicons/ionicons5'
import type { CustomerWithLatestTrack, NextAction, CustomerGroup } from '@/types'

interface Props {
  customer: CustomerWithLatestTrack
}

interface Emits {
  (e: 'view', customer: CustomerWithLatestTrack): void
  (e: 'edit', customer: CustomerWithLatestTrack): void
  (e: 'delete', customer: CustomerWithLatestTrack): void
  (e: 'track', customer: CustomerWithLatestTrack): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

// 下拉菜单选项
const dropdownOptions = computed(() => [
  {
    label: '查看详情',
    key: 'view',
    icon: () => h(EyeOutline)
  },
  {
    label: '编辑客户',
    key: 'edit',
    icon: () => h(CreateOutline)
  },
  {
    label: '添加跟进',
    key: 'track',
    icon: () => h(AddCircleOutline)
  },
  {
    type: 'divider',
    key: 'divider'
  },
  {
    label: '删除客户',
    key: 'delete',
    icon: () => h(TrashOutline),
    props: {
      style: 'color: #ff6b6b;'
    }
  }
])

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

// 获取分组标签类型
const getGroupTagType = (group: CustomerGroup) => {
  switch (group) {
    case '团课':
      return 'info'
    case '小班':
      return 'success'
    case '私教':
      return 'warning'
    case '教培':
      return 'error'
    default:
      return 'default'
  }
}

// 处理菜单选择
const handleMenuSelect = (key: string) => {
  switch (key) {
    case 'view':
      emit('view', props.customer)
      break
    case 'edit':
      emit('edit', props.customer)
      break
    case 'track':
      emit('track', props.customer)
      break
    case 'delete':
      emit('delete', props.customer)
      break
  }
}

// 处理拨打电话
const handleCall = () => {
  if (props.customer.phone) {
    window.open(`tel:${props.customer.phone}`)
  }
}

// 文本截断
const truncateText = (text: string, length: number) => {
  return text.length > length ? text.substring(0, length) + '...' : text
}

// 格式化日期
const formatDate = (dateString: string) => {
  const date = new Date(dateString)
  const now = new Date()
  const diffTime = now.getTime() - date.getTime()
  const diffDays = Math.ceil(diffTime / (1000 * 60 * 60 * 24))
  
  if (diffDays === 1) {
    return '昨天'
  } else if (diffDays <= 7) {
    return `${diffDays}天前`
  } else if (diffDays <= 30) {
    return `${Math.ceil(diffDays / 7)}周前`
  } else {
    return date.toLocaleDateString('zh-CN', {
      month: 'short',
      day: 'numeric'
    })
  }
}
</script>

<style scoped>
.customer-card {
  transition: all 0.2s ease;
  cursor: pointer;
  border: 1px solid #e0e0e0;
}

.customer-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.1);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 8px;
}

.customer-info {
  flex: 1;
  min-width: 0;
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 8px;
}

.customer-name {
  font-size: 16px;
  font-weight: 600;
  color: #333;
  flex: 1;
  min-width: 0;
  word-break: break-word;
  line-height: 1.3;
}

.status-tags {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
}

.group-tag {
  font-weight: 500;
}

.status-tag {
  font-weight: 500;
}

.card-content {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.contact-info {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.info-item {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  min-height: 20px;
}

.info-item .n-icon {
  color: #666;
  flex-shrink: 0;
}

.email-text,
.address-text {
  word-break: break-all;
  line-height: 1.3;
}

.action-btn {
  margin-left: auto;
  font-size: 11px;
}

.notes-section {
  padding: 8px;
  background: #f8f9fa;
  border-radius: 6px;
  border-left: 3px solid #1890ff;
}

.notes-text {
  font-size: 12px;
  line-height: 1.4;
  word-break: break-word;
}

.stats-section {
  display: flex;
  justify-content: space-between;
  padding: 8px;
  background: #fafafa;
  border-radius: 6px;
}

.stat-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
}

.last-track-section {
  padding: 10px;
  background: linear-gradient(135deg, #f6f8fa 0%, #f1f3f4 100%);
  border-radius: 8px;
  border-left: 3px solid #52c41a;
}

/* 移动端适配 */
@media (max-width: 768px) {
  .customer-info {
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
  }
  
  .status-tags {
    align-self: flex-end;
    gap: 4px;
  }
  
  .customer-name {
    margin-bottom: 4px;
  }
}

@media (max-width: 480px) {
  .status-tags {
    flex-direction: column;
    gap: 3px;
    align-items: flex-end;
  }
  
  .group-tag,
  .status-tag {
    font-size: 11px;
  }
}

.last-track-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 6px;
}

.last-track-content {
  padding-left: 8px;
  border-left: 2px solid #e1e4e8;
  line-height: 1.4;
}

.footer-info {
  text-align: center;
  padding-top: 8px;
  border-top: 1px solid #f0f0f0;
}

/* 移动端适配 */
@media (max-width: 480px) {
  .customer-name {
    font-size: 15px;
  }
  
  .info-item {
    font-size: 12px;
  }
  
  .stats-section {
    flex-direction: column;
    gap: 8px;
  }
  
  .stat-item {
    flex-direction: row;
    justify-content: space-between;
  }
}
</style>
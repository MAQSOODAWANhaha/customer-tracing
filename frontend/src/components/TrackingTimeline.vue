<template>
  <n-card title="跟进记录" class="timeline-card">
    <template #header-extra>
      <n-space size="small">
        <n-button 
          size="small" 
          @click="handleRefresh"
          :loading="loading"
        >
          <template #icon>
            <n-icon :component="RefreshOutline" />
          </template>
          刷新
        </n-button>
        <n-button 
          type="primary" 
          size="small"
          @click="$emit('add-track')"
        >
          <template #icon>
            <n-icon :component="AddOutline" />
          </template>
          添加
        </n-button>
      </n-space>
    </template>

    <n-spin :show="loading">
      <div v-if="tracks.length === 0 && !loading" class="empty-timeline">
        <n-empty description="暂无跟进记录">
          <template #extra>
            <n-button 
              type="primary" 
              @click="$emit('add-track')"
            >
              添加第一条跟进记录
            </n-button>
          </template>
        </n-empty>
      </div>

      <n-timeline v-else class="timeline">
        <n-timeline-item
          v-for="track in tracks"
          :key="track.id"
          :type="getTimelineType(track.next_action)"
          :title="formatTimelineTitle(track)"
          :time="formatTime(track.created_at)"
        >
          <template #icon>
            <n-icon 
              :component="getTimelineIcon(track.next_action)" 
              size="16"
            />
          </template>

          <div class="timeline-content">
            <!-- 跟进内容 -->
            <div class="track-content">
              <n-text>{{ track.content }}</n-text>
            </div>

            <!-- 下一步行动 -->
            <div class="track-meta">
              <n-tag 
                :type="getStatusTagType(track.next_action)" 
                size="small"
                round
              >
                {{ track.next_action }}
              </n-tag>
              
              <!-- 操作按钮 -->
              <n-space size="small" class="track-actions">
                <n-button 
                  text 
                  size="tiny"
                  @click="handleEdit(track)"
                >
                  <template #icon>
                    <n-icon :component="CreateOutline" />
                  </template>
                  编辑
                </n-button>
                <n-button 
                  text 
                  size="tiny"
                  type="error"
                  @click="handleDelete(track)"
                >
                  <template #icon>
                    <n-icon :component="TrashOutline" />
                  </template>
                  删除
                </n-button>
              </n-space>
            </div>
          </div>
        </n-timeline-item>
      </n-timeline>
    </n-spin>

    <!-- 分页 -->
    <div class="pagination-container" v-if="totalCount > pageSize">
      <n-pagination
        v-model:page="currentPage"
        :page-count="Math.ceil(totalCount / pageSize)"
        :page-size="pageSize"
        size="small"
        show-size-picker
        :page-sizes="[10, 20, 50]"
        @update:page="handlePageChange"
        @update:page-size="handlePageSizeChange"
      />
    </div>

    <!-- 编辑跟进记录弹窗 -->
    <track-form-modal
      v-model:show="showEditModal"
      :track="editingTrack"
      :customer="{ id: customerId }"
      @success="handleEditSuccess"
    />

    <!-- 删除确认弹窗 -->
    <n-modal v-model:show="showDeleteModal" preset="dialog" title="确认删除">
      <template #header>
        <div style="display: flex; align-items: center;">
          <n-icon :component="TrashOutline" color="#ff6b6b" size="20" style="margin-right: 8px;" />
          确认删除
        </div>
      </template>
      
      <div style="margin: 16px 0;">
        确定要删除这条跟进记录吗？
        <br />
        <n-text depth="3" style="font-size: 12px;">
          删除后将无法恢复。
        </n-text>
      </div>
      
      <template #action>
        <n-space>
          <n-button @click="showDeleteModal = false">取消</n-button>
          <n-button 
            type="error" 
            @click="confirmDelete"
            :loading="deleting"
          >
            确定删除
          </n-button>
        </n-space>
      </template>
    </n-modal>
  </n-card>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useMessage } from 'naive-ui'
import {
  RefreshOutline,
  AddOutline,
  CreateOutline,
  TrashOutline,
  CheckmarkCircleOutline,
  CloseCircleOutline,
  TimeOutline
} from '@vicons/ionicons5'
import { useTrackStore } from '@/stores/track'
import TrackFormModal from '@/components/TrackFormModal.vue'
import type { CustomerTrack, NextAction } from '@/types'

interface Props {
  customerId: number
}

interface Emits {
  (e: 'refresh'): void
  (e: 'add-track'): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const message = useMessage()
const trackStore = useTrackStore()

const loading = ref(false)
const currentPage = ref(1)
const pageSize = ref(20)
const showEditModal = ref(false)
const showDeleteModal = ref(false)
const editingTrack = ref<CustomerTrack | null>(null)
const deletingTrack = ref<CustomerTrack | null>(null)
const deleting = ref(false)

// 计算属性
const tracks = computed(() => trackStore.tracks)
const totalCount = computed(() => trackStore.totalCount)

// 加载跟进记录
const loadTracks = async () => {
  try {
    loading.value = true
    await trackStore.fetchTracks({
      customer_id: props.customerId,
      page: currentPage.value,
      limit: pageSize.value
    })
  } catch (error: any) {
    message.error(error.message || '加载跟进记录失败')
  } finally {
    loading.value = false
  }
}

// 获取时间线类型
const getTimelineType = (nextAction: NextAction) => {
  switch (nextAction) {
    case '继续跟进':
      return 'success'
    case '结束跟进':
      return 'default'
    default:
      return 'info'
  }
}

// 获取时间线图标
const getTimelineIcon = (nextAction: NextAction) => {
  switch (nextAction) {
    case '继续跟进':
      return CheckmarkCircleOutline
    case '结束跟进':
      return CloseCircleOutline
    default:
      return TimeOutline
  }
}

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

// 格式化时间线标题
const formatTimelineTitle = (track: CustomerTrack) => {
  const date = new Date(track.created_at)
  return date.toLocaleDateString('zh-CN', {
    month: 'short',
    day: 'numeric',
    weekday: 'short'
  })
}

// 格式化时间
const formatTime = (dateString: string) => {
  const date = new Date(dateString)
  const now = new Date()
  const diffTime = now.getTime() - date.getTime()
  const diffDays = Math.floor(diffTime / (1000 * 60 * 60 * 24))
  const diffHours = Math.floor(diffTime / (1000 * 60 * 60))
  const diffMinutes = Math.floor(diffTime / (1000 * 60))

  if (diffMinutes < 1) {
    return '刚刚'
  } else if (diffMinutes < 60) {
    return `${diffMinutes}分钟前`
  } else if (diffHours < 24) {
    return `${diffHours}小时前`
  } else if (diffDays === 1) {
    return '昨天 ' + date.toLocaleTimeString('zh-CN', { 
      hour: '2-digit', 
      minute: '2-digit' 
    })
  } else if (diffDays < 7) {
    return `${diffDays}天前`
  } else {
    return date.toLocaleString('zh-CN', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    })
  }
}

// 处理操作
const handleRefresh = () => {
  loadTracks()
  emit('refresh')
}

const handlePageChange = (page: number) => {
  currentPage.value = page
  loadTracks()
}

const handlePageSizeChange = (size: number) => {
  pageSize.value = size
  currentPage.value = 1
  loadTracks()
}

const handleEdit = (track: CustomerTrack) => {
  editingTrack.value = track
  showEditModal.value = true
}

const handleDelete = (track: CustomerTrack) => {
  deletingTrack.value = track
  showDeleteModal.value = true
}

const handleEditSuccess = () => {
  showEditModal.value = false
  editingTrack.value = null
  loadTracks()
  emit('refresh')
  message.success('跟进记录更新成功')
}

const confirmDelete = async () => {
  if (!deletingTrack.value) return
  
  try {
    deleting.value = true
    await trackStore.deleteTrack(deletingTrack.value.id)
    message.success('删除成功')
    showDeleteModal.value = false
    deletingTrack.value = null
    loadTracks()
    emit('refresh')
  } catch (error: any) {
    message.error(error.message || '删除失败')
  } finally {
    deleting.value = false
  }
}

// 监听客户ID变化
watch(
  () => props.customerId,
  (newId) => {
    if (newId) {
      currentPage.value = 1
      loadTracks()
    }
  }
)

// 页面加载
onMounted(() => {
  if (props.customerId) {
    loadTracks()
  }
})

// 暴露方法给父组件调用
defineExpose({
  loadTracks,
  handleRefresh
})
</script>

<style scoped>
.timeline-card {
  margin-bottom: 16px;
}

.empty-timeline {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 200px;
}

.timeline {
  margin-top: 16px;
}

.timeline-content {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.track-content {
  line-height: 1.5;
  word-break: break-word;
}

.track-meta {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 8px;
}

.track-actions {
  opacity: 0.7;
  transition: opacity 0.2s ease;
}

.timeline-content:hover .track-actions {
  opacity: 1;
}

.pagination-container {
  display: flex;
  justify-content: center;
  margin-top: 20px;
  padding-top: 16px;
  border-top: 1px solid #f0f0f0;
}

/* 移动端适配 */
@media (max-width: 768px) {
  .track-meta {
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
  }
  
  .track-actions {
    opacity: 1;
  }
}
</style>
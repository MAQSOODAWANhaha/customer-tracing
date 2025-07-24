<template>
  <n-modal 
    v-model:show="showModal" 
    preset="card" 
    :title="isEditing ? '编辑跟进记录' : '添加跟进记录'"
    class="track-form-modal"
    size="large"
    :bordered="false"
    :segmented="{ content: true }"
    :close-on-esc="false"
    :mask-closable="false"
  >

    <div v-if="customer" class="customer-info">
      <n-text depth="2">客户：</n-text>
      <n-text strong>{{ customer.name }}</n-text>
    </div>

    <n-form
      ref="formRef"
      :model="formData"
      :rules="formRules"
      label-placement="left"
      label-width="auto"
      require-mark-placement="right-hanging"
      size="medium"
    >
      <n-form-item label="跟进内容" path="content">
        <n-input
          v-model:value="formData.content"
          type="textarea"
          placeholder="请详细描述本次跟进的内容，如沟通要点、客户反馈、达成的共识等..."
          :rows="6"
          clearable
          maxlength="2000"
          show-count
        />
      </n-form-item>
      
      <n-form-item label="下一步行动" path="next_action">
        <n-radio-group v-model:value="formData.next_action">
          <n-space>
            <n-radio value="继续跟进">
              <n-space align="center" size="small">
                <n-icon :component="CheckmarkCircleOutline" color="#52c41a" />
                <span>继续跟进</span>
              </n-space>
            </n-radio>
            <n-radio value="结束跟进">
              <n-space align="center" size="small">
                <n-icon :component="CloseCircleOutline" color="#999" />
                <span>结束跟进</span>
              </n-space>
            </n-radio>
          </n-space>
        </n-radio-group>
      </n-form-item>

      <!-- 下一步行动说明 -->
      <n-form-item v-if="formData.next_action">
        <n-alert 
          :type="formData.next_action === '继续跟进' ? 'success' : 'info'"
          :show-icon="false"
          closable
        >
          <template #header>
            {{ formData.next_action === '继续跟进' ? '继续跟进' : '结束跟进' }}
          </template>
          {{ getActionDescription(formData.next_action) }}
        </n-alert>
      </n-form-item>

      <!-- 快捷模板 -->
      <n-form-item label="常用模板">
        <n-space>
          <n-button 
            v-for="template in commonTemplates"
            :key="template.id"
            size="small"
            secondary
            @click="applyTemplate(template)"
          >
            {{ template.name }}
          </n-button>
        </n-space>
      </n-form-item>
    </n-form>

    <template #action>
      <n-space>
        <n-button @click="handleCancel" :disabled="loading">
          取消
        </n-button>
        <n-button 
          type="primary" 
          @click="handleSubmit"
          :loading="loading"
        >
          {{ isEditing ? '保存' : '添加' }}
        </n-button>
      </n-space>
    </template>
  </n-modal>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch } from 'vue'
import { useMessage, type FormInst, type FormRules } from 'naive-ui'
import { 
  CheckmarkCircleOutline,
  CloseCircleOutline
} from '@vicons/ionicons5'
import { useTrackStore } from '@/stores/track'
import type { 
  CustomerTrack, 
  TrackCreateRequest, 
  TrackUpdateRequest, 
  NextAction 
} from '@/types'

interface Props {
  show: boolean
  customer?: { id: number; name?: string } | null
  track?: CustomerTrack | null
}

interface Emits {
  (e: 'update:show', value: boolean): void
  (e: 'success'): void
}

interface TrackTemplate {
  id: string
  name: string
  content: string
  next_action: NextAction
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const message = useMessage()
const trackStore = useTrackStore()

const formRef = ref<FormInst | null>(null)
const loading = ref(false)

// 表单数据
const formData = reactive<TrackCreateRequest & { id?: number }>({
  customer_id: 0,
  content: '',
  next_action: '继续跟进'
})

// 计算属性
const showModal = computed({
  get: () => props.show,
  set: (value) => emit('update:show', value)
})

const isEditing = computed(() => !!props.track?.id)

// 常用模板
const commonTemplates: TrackTemplate[] = [
  {
    id: 'initial_contact',
    name: '初次接触',
    content: '今日首次与客户取得联系，介绍了我们的产品/服务特点，客户表现出一定兴趣。',
    next_action: '继续跟进'
  },
  {
    id: 'needs_analysis',
    name: '需求分析',
    content: '深入了解客户需求，确认了具体的痛点和期望解决的问题。',
    next_action: '继续跟进'
  },
  {
    id: 'proposal_sent',
    name: '方案提交',
    content: '已向客户提交详细的解决方案和报价，等待客户反馈。',
    next_action: '继续跟进'
  },
  {
    id: 'follow_up',
    name: '定期跟进',
    content: '与客户保持联系，了解项目进展和决策时间。',
    next_action: '继续跟进'
  },
  {
    id: 'deal_closed',
    name: '成功签约',
    content: '客户已确认合作，签署了正式合同。',
    next_action: '结束跟进'
  },
  {
    id: 'lost_deal',
    name: '暂停跟进',
    content: '客户暂时没有合作需求，或选择了其他供应商。',
    next_action: '结束跟进'
  }
]

// 表单验证规则
const formRules: FormRules = {
  content: [
    { required: true, message: '请输入跟进内容', trigger: ['input', 'blur'] },
    { min: 10, message: '跟进内容至少需要 10 个字符', trigger: ['input', 'blur'] },
    { max: 2000, message: '跟进内容不能超过 2000 个字符', trigger: ['input', 'blur'] }
  ],
  next_action: [
    { required: true, message: '请选择下一步行动', trigger: ['change', 'blur'] }
  ]
}

// 获取行动描述
const getActionDescription = (action: NextAction) => {
  switch (action) {
    case '继续跟进':
      return '该客户仍有合作可能，建议继续保持联系和跟进。'
    case '结束跟进':
      return '该客户暂无合作需求或已完成合作，可以结束跟进。'
    default:
      return ''
  }
}

// 应用模板
const applyTemplate = (template: TrackTemplate) => {
  formData.content = template.content
  formData.next_action = template.next_action
}

// 重置表单
const resetForm = () => {
  formData.content = ''
  formData.next_action = '继续跟进'
  delete formData.id
  
  if (props.customer?.id) {
    formData.customer_id = props.customer.id
  }
}

// 加载跟进记录数据
const loadTrackData = (track: CustomerTrack) => {
  formData.id = track.id
  formData.customer_id = track.customer_id
  formData.content = track.content
  formData.next_action = track.next_action
}

// 处理提交
const handleSubmit = async () => {
  if (!formRef.value) return
  
  try {
    await formRef.value.validate()
    loading.value = true
    
    if (isEditing.value && formData.id) {
      // 编辑跟进记录
      const updateData: TrackUpdateRequest = {
        content: formData.content,
        next_action: formData.next_action
      }
      await trackStore.updateTrack(formData.id, updateData)
      message.success('跟进记录更新成功')
    } else {
      // 新增跟进记录
      const createData: TrackCreateRequest = {
        customer_id: formData.customer_id,
        content: formData.content,
        next_action: formData.next_action
      }
      await trackStore.createTrack(createData)
      message.success('跟进记录添加成功')
    }
    
    emit('success')
  } catch (error: any) {
    message.error(error.message || (isEditing.value ? '更新失败' : '添加失败'))
  } finally {
    loading.value = false
  }
}

// 处理取消
const handleCancel = () => {
  showModal.value = false
}

// 监听弹窗显示状态
watch(
  () => props.show,
  (show) => {
    if (show) {
      if (props.track) {
        loadTrackData(props.track)
      } else {
        resetForm()
      }
    }
  }
)

// 监听跟进记录变化
watch(
  () => props.track,
  (track) => {
    if (track && props.show) {
      loadTrackData(track)
    }
  }
)

// 监听客户变化
watch(
  () => props.customer,
  (customer) => {
    if (customer && !props.track) {
      formData.customer_id = customer.id
    }
  },
  { immediate: true }
)
</script>

<style scoped>
.track-form-modal {
  max-width: 700px;
}

.track-form-modal :deep(.n-card-header) {
  border-bottom: 1px solid #f0f0f0;
}

.track-form-modal :deep(.n-card__action) {
  border-top: 1px solid #f0f0f0;
  padding-top: 16px;
}

.customer-info {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 16px;
  background: #f8f9fa;
  border-radius: 6px;
  margin-bottom: 16px;
  border-left: 3px solid #1890ff;
}

/* 移动端适配 */
@media (max-width: 768px) {
  .track-form-modal {
    margin: 16px;
    max-width: none;
  }
  
  .track-form-modal :deep(.n-card) {
    border-radius: 8px;
  }
}
</style>
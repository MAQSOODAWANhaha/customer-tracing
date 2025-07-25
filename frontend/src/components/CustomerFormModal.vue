<template>
  <n-modal 
    v-model:show="showModal" 
    preset="card" 
    :title="isEditing ? '编辑客户' : '新增客户'"
    class="customer-form-modal"
    size="large"
    :bordered="false"
    :segmented="{ content: true }"
    :close-on-esc="false"
    :mask-closable="false"
  >

    <n-form
      ref="formRef"
      :model="formData"
      :rules="formRules"
      label-placement="left"
      label-width="auto"
      require-mark-placement="right-hanging"
      size="medium"
    >
      <n-grid :cols="24" :x-gap="16">
        <!-- 基本信息 -->
        <n-form-item-gi :span="24" label="客户姓名" path="name">
          <n-input
            v-model:value="formData.name"
            placeholder="请输入客户姓名"
            clearable
            maxlength="100"
            show-count
          />
        </n-form-item-gi>
        
        <n-form-item-gi :span="12" label="手机号码" path="phone">
          <n-input
            v-model:value="formData.phone"
            placeholder="请输入手机号码"
            clearable
            maxlength="20"
          />
        </n-form-item-gi>
        
        <n-form-item-gi :span="12" label="客户分组" path="customer_group">
          <n-select
            v-model:value="formData.customer_group"
            placeholder="请选择客户分组"
            :options="customerGroupOptions"
          />
        </n-form-item-gi>
        
        <n-form-item-gi :span="12" label="客户评级" path="rate">
          <div class="rate-container">
            <n-rate
              v-model:value="formData.rate"
              :max="5"
              allow-half
              clearable
              class="rate-stars"
            />
            <n-text depth="3" class="rate-text">
              {{ formData.rate || 0 }}/5
            </n-text>
          </div>
        </n-form-item-gi>
        
        <n-form-item-gi :span="24" label="联系地址" path="address">
          <n-input
            v-model:value="formData.address"
            placeholder="请输入联系地址"
            clearable
            maxlength="500"
          />
        </n-form-item-gi>
        
        <n-form-item-gi :span="24" label="备注信息" path="notes">
          <n-input
            v-model:value="formData.notes"
            type="textarea"
            placeholder="请输入备注信息，如客户偏好、特殊要求等"
            :rows="4"
            clearable
            maxlength="1000"
            show-count
          />
        </n-form-item-gi>
      </n-grid>
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
          {{ isEditing ? '保存' : '创建' }}
        </n-button>
      </n-space>
    </template>
  </n-modal>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch } from 'vue'
import { useMessage, type FormInst, type FormRules } from 'naive-ui'
import { useCustomerStore } from '@/stores/customer'
import type { Customer, CustomerCreateRequest, CustomerUpdateRequest, CustomerGroup } from '@/types'

interface Props {
  show: boolean
  customer?: Customer | null
}

interface Emits {
  (e: 'update:show', value: boolean): void
  (e: 'success'): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const message = useMessage()
const customerStore = useCustomerStore()

const formRef = ref<FormInst | null>(null)
const loading = ref(false)

// 表单数据
const formData = reactive<CustomerCreateRequest & { id?: number }>({
  name: '',
  phone: '',
  address: '',
  notes: '',
  rate: 0,
  customer_group: '团课' as CustomerGroup
})

// 客户分组选项
const customerGroupOptions = [
  { label: '团课', value: '团课' as CustomerGroup },
  { label: '小班', value: '小班' as CustomerGroup },
  { label: '私教', value: '私教' as CustomerGroup },
  { label: '教培', value: '教培' as CustomerGroup }
]

// 计算属性
const showModal = computed({
  get: () => props.show,
  set: (value) => emit('update:show', value)
})

const isEditing = computed(() => !!props.customer?.id)

// 表单验证规则
const formRules: FormRules = {
  name: [
    { required: true, message: '请输入客户姓名', trigger: ['input', 'blur'] },
    { min: 2, max: 100, message: '客户姓名长度为 2-100 个字符', trigger: ['input', 'blur'] }
  ],
  phone: [
    { 
      pattern: /^1[3-9]\d{9}$/, 
      message: '请输入正确的手机号码', 
      trigger: ['input', 'blur'] 
    }
  ],
  address: [
    { max: 500, message: '联系地址不能超过 500 个字符', trigger: ['input', 'blur'] }
  ],
  notes: [
    { max: 1000, message: '备注信息不能超过 1000 个字符', trigger: ['input', 'blur'] }
  ],
  rate: [
    { type: 'number', min: 0, max: 5, message: '评级必须在 0-5 之间', trigger: ['change', 'blur'] }
  ],
  customer_group: [
    { required: true, message: '请选择客户分组', trigger: ['change', 'blur'] }
  ]
}

// 重置表单
const resetForm = () => {
  formData.name = ''
  formData.phone = ''
  formData.address = ''
  formData.notes = ''
  formData.rate = 0
  formData.customer_group = '团课'
  delete formData.id
}

// 加载客户数据
const loadCustomerData = (customer: Customer) => {
  formData.id = customer.id
  formData.name = customer.name
  formData.phone = customer.phone || ''
  formData.address = customer.address || ''
  formData.notes = customer.notes || ''
  formData.rate = customer.rate || 0
  formData.customer_group = customer.customer_group || '团课'
}

// 处理提交
const handleSubmit = async () => {
  if (!formRef.value) return
  
  try {
    await formRef.value.validate()
    loading.value = true
    
    if (isEditing.value && formData.id) {
      // 编辑客户
      const updateData: CustomerUpdateRequest = {
        name: formData.name,
        phone: formData.phone || null,
        address: formData.address || null,
        notes: formData.notes || null,
        rate: formData.rate,
        customer_group: formData.customer_group
      }
      await customerStore.updateCustomer(formData.id, updateData)
      message.success('客户信息更新成功')
    } else {
      // 新增客户
      const createData: CustomerCreateRequest = {
        name: formData.name,
        phone: formData.phone || null,
        address: formData.address || null,
        notes: formData.notes || null,
        rate: formData.rate,
        customer_group: formData.customer_group
      }
      await customerStore.createCustomer(createData)
      message.success('客户创建成功')
    }
    
    emit('success')
  } catch (error: any) {
    message.error(error.message || (isEditing.value ? '更新失败' : '创建失败'))
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
      if (props.customer) {
        loadCustomerData(props.customer)
      } else {
        resetForm()
      }
    }
  }
)

// 监听客户变化
watch(
  () => props.customer,
  (customer) => {
    if (customer && props.show) {
      loadCustomerData(customer)
    }
  }
)
</script>

<style scoped>
.customer-form-modal {
  max-width: 600px;
}

.customer-form-modal :deep(.n-card-header) {
  border-bottom: 1px solid #f0f0f0;
  font-size: 18px;
  font-weight: 600;
  padding: 20px 24px 16px 24px;
}

.customer-form-modal :deep(.n-card__content) {
  padding: 24px;
}

.customer-form-modal :deep(.n-card__action) {
  border-top: 1px solid #f0f0f0;
  padding: 16px 24px 20px 24px;
}

/* 评级容器样式 */
.rate-container {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: nowrap;
}

.rate-text {
  font-size: 12px;
  color: #666;
  white-space: nowrap;
  flex-shrink: 0;
}

/* 平板适配 */
@media (max-width: 1024px) {
  .customer-form-modal {
    max-width: 90%;
    margin: 20px auto;
  }
}

/* 移动端适配 */
@media (max-width: 768px) {
  .customer-form-modal {
    margin: 8px;
    max-width: none;
    width: calc(100vw - 16px);
    max-height: calc(100vh - 16px);
  }
  
  .customer-form-modal :deep(.n-card) {
    border-radius: 12px;
    height: auto;
    max-height: calc(100vh - 16px);
    display: flex;
    flex-direction: column;
  }

  .customer-form-modal :deep(.n-card__content) {
    padding: 20px;
    flex: 1;
    overflow-y: auto;
  }
  
  .customer-form-modal :deep(.n-card-header) {
    padding: 20px 20px 16px 20px;
    font-size: 18px;
    flex-shrink: 0;
  }
  
  .customer-form-modal :deep(.n-card__action) {
    padding: 16px 20px;
    flex-shrink: 0;
  }

  /* 表单在移动端改为单列布局 */
  .customer-form-modal :deep(.n-grid) {
    display: block !important;
  }
  
  .customer-form-modal :deep(.n-form-item-gi) {
    width: 100% !important;
    margin-bottom: 20px;
  }

  /* 移动端评级优化 */
  .rate-container {
    justify-content: flex-start;
    gap: 12px;
  }

  .rate-stars :deep(.n-rate) {
    font-size: 18px;
  }

  .rate-text {
    font-size: 14px;
  }

  /* 移动端按钮优化 */
  .customer-form-modal :deep(.n-space) {
    width: 100%;
    justify-content: stretch;
  }
  
  .customer-form-modal :deep(.n-button) {
    flex: 1;
    height: 44px;
    font-size: 16px;
    font-weight: 500;
  }
}

@media (max-width: 480px) {
  .customer-form-modal {
    margin: 4px;
    width: calc(100vw - 8px);
    max-height: calc(100vh - 8px);
  }
  
  .customer-form-modal :deep(.n-card) {
    border-radius: 8px;
  }

  .customer-form-modal :deep(.n-card__content) {
    padding: 16px;
  }
  
  .customer-form-modal :deep(.n-card-header) {
    padding: 16px 16px 12px 16px;
    font-size: 17px;
  }
  
  .customer-form-modal :deep(.n-card__action) {
    padding: 12px 16px 16px 16px;
  }

  .customer-form-modal :deep(.n-form-item-gi) {
    margin-bottom: 16px;
  }

  /* 小屏按钮堆叠布局 */
  .customer-form-modal :deep(.n-space) {
    flex-direction: column;
    gap: 8px !important;
  }
  
  .customer-form-modal :deep(.n-button) {
    width: 100%;
    height: 48px;
  }
}
</style>
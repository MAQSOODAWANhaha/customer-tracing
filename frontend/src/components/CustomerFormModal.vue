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
    <template #header-extra>
      <n-button text @click="handleCancel">
        <template #icon>
          <n-icon :component="CloseOutline" />
        </template>
      </n-button>
    </template>

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
        
        <n-form-item-gi :span="12" label="客户评级" path="rate">
          <n-rate
            v-model:value="formData.rate"
            :max="5"
            allow-half
            clearable
          />
          <n-text depth="3" style="font-size: 12px; margin-left: 8px;">
            {{ formData.rate || 0 }}/5
          </n-text>
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
import { CloseOutline } from '@vicons/ionicons5'
import { useCustomerStore } from '@/stores/customer'
import type { Customer, CustomerCreateRequest, CustomerUpdateRequest } from '@/types'

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
  rate: 0
})

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
  ]
}

// 重置表单
const resetForm = () => {
  formData.name = ''
  formData.phone = ''
  formData.address = ''
  formData.notes = ''
  formData.rate = 0
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
        rate: formData.rate
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
        rate: formData.rate
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
}

.customer-form-modal :deep(.n-card__action) {
  border-top: 1px solid #f0f0f0;
  padding-top: 16px;
}

/* 移动端适配 */
@media (max-width: 768px) {
  .customer-form-modal {
    margin: 16px;
    max-width: none;
  }
  
  .customer-form-modal :deep(.n-card) {
    border-radius: 8px;
  }
}
</style>
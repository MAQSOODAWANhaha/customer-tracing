<template>
  <div class="customer-list-container">
    <!-- 页面头部 -->
    <n-card class="header-card" :bordered="false">
      <div class="header-content">
        <div class="header-info">
          <h1>客户管理</h1>
          <n-text depth="3">管理您的客户信息和跟进记录</n-text>
        </div>
        <n-space>
          <n-button 
            type="primary" 
            @click="showCreateModal = true"
            :loading="loading"
          >
            <template #icon>
              <n-icon :component="AddOutline" />
            </template>
            新增客户
          </n-button>
          <n-button @click="handleRefresh" :loading="loading">
            <template #icon>
              <n-icon :component="RefreshOutline" />
            </template>
            刷新
          </n-button>
          <n-button text @click="handleLogout">
            <template #icon>
              <n-icon :component="LogOutOutline" />
            </template>
            退出登录
          </n-button>
        </n-space>
      </div>
    </n-card>

    <!-- 搜索和筛选 -->
    <n-card class="filter-card" :bordered="false">
      <div class="filter-content">
        <!-- 搜索栏 -->
        <div class="search-section">
          <n-input
            v-model:value="searchQuery"
            placeholder="搜索客户姓名、手机号..."
            clearable
            class="search-input"
            @input="handleSearch"
          >
            <template #prefix>
              <n-icon :component="SearchOutline" />
            </template>
          </n-input>
        </div>
        
        <!-- 筛选和统计区域 -->
        <div class="filter-section">
          <div class="filter-controls">
            <n-select
              v-model:value="statusFilter"
              placeholder="筛选状态"
              clearable
              class="filter-select"
              :options="statusOptions"
              @update:value="handleFilterChange"
            />

            <n-select
              v-model:value="groupFilter"
              placeholder="客户分组"
              clearable
              class="filter-select"
              :options="groupOptions"
              @update:value="handleFilterChange"
            />
          </div>
          
          <div class="stats-section">
            <n-text depth="3" class="stats-text">
              共 {{ totalCount }} 位客户
            </n-text>
          </div>
        </div>
      </div>
    </n-card>

    <!-- 客户列表 -->
    <n-card class="list-card" :bordered="false">
      <n-spin :show="loading">
        <div v-if="customers.length === 0 && !loading" class="empty-state">
          <n-empty description="暂无客户数据">
            <template #extra>
              <n-button 
                type="primary" 
                @click="showCreateModal = true"
              >
                添加第一个客户
              </n-button>
            </template>
          </n-empty>
        </div>
        
        <div v-else class="customer-grid">
          <customer-card
            v-for="customer in customers"
            :key="customer.id"
            :customer="customer"
            @view="handleViewCustomer"
            @edit="handleEditCustomer"
            @delete="handleDeleteCustomer"
            @track="handleTrackCustomer"
          />
        </div>
      </n-spin>
    </n-card>

    <!-- 分页 -->
    <div class="pagination-container" v-if="totalCount > pageSize">
      <n-pagination
        v-model:page="currentPage"
        :page-count="Math.ceil(totalCount / pageSize)"
        :page-size="pageSize"
        show-size-picker
        :page-sizes="[10, 20, 50]"
        show-quick-jumper
        @update:page="handlePageChange"
        @update:page-size="handlePageSizeChange"
      />
    </div>

    <!-- 新增/编辑客户弹窗 -->
    <customer-form-modal
      v-model:show="showCreateModal"
      :customer="editingCustomer"
      @success="handleFormSuccess"
    />

    <!-- 删除确认弹窗 -->
    <n-modal v-model:show="showDeleteModal" preset="dialog" title="确认删除" class="delete-modal">
      <template #header>
        <div style="display: flex; align-items: center;">
          <n-icon :component="TrashOutline" color="#ff6b6b" size="20" style="margin-right: 8px;" />
          确认删除
        </div>
      </template>
      
      <div style="margin: 16px 0;">
        确定要删除客户 <strong>{{ deletingCustomer?.name }}</strong> 吗？
        <br />
        <n-text depth="3" style="font-size: 12px;">
          删除后将无法恢复，该客户的所有跟进记录也将被删除。
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
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useMessage } from 'naive-ui'
import { 
  AddOutline, 
  RefreshOutline, 
  SearchOutline, 
  LogOutOutline,
  TrashOutline
} from '@vicons/ionicons5'
import { useAuthStore } from '@/stores/auth'
import { useCustomerStore } from '@/stores/customer'
import CustomerCard from '@/components/CustomerCard.vue'
import CustomerFormModal from '@/components/CustomerFormModal.vue'
import type { Customer, NextAction, CustomerGroup } from '@/types'

const router = useRouter()
const message = useMessage()
const authStore = useAuthStore()
const customerStore = useCustomerStore()

// 响应式数据
const loading = ref(false)
const searchQuery = ref('')
const statusFilter = ref<NextAction | null>(null)
const groupFilter = ref<CustomerGroup | null>(null)
const currentPage = ref(1)
const pageSize = ref(20)

// 弹窗状态
const showCreateModal = ref(false)
const showDeleteModal = ref(false)
const editingCustomer = ref<Customer | null>(null)
const deletingCustomer = ref<Customer | null>(null)
const deleting = ref(false)

// 计算属性
const customers = computed(() => customerStore.customers)
const totalCount = computed(() => customerStore.totalCount)

// 状态选项
const statusOptions = [
  { label: '继续跟进', value: '继续跟进' as NextAction },
  { label: '结束跟进', value: '结束跟进' as NextAction }
]

// 分组选项
const groupOptions = [
  { label: '团课', value: '团课' as CustomerGroup },
  { label: '小班', value: '小班' as CustomerGroup },
  { label: '私教', value: '私教' as CustomerGroup },
  { label: '教培', value: '教培' as CustomerGroup }
]

// 搜索防抖
let searchTimeout: NodeJS.Timeout | null = null

// 加载客户列表
const loadCustomers = async () => {
  try {
    loading.value = true
    await customerStore.fetchCustomers({
      page: currentPage.value,
      limit: pageSize.value,
      search: searchQuery.value || undefined,
      status: statusFilter.value || undefined,
      customer_group: groupFilter.value || undefined
    })
  } catch (error: any) {
    message.error(error.message || '加载客户列表失败')
  } finally {
    loading.value = false
  }
}

// 搜索处理
const handleSearch = () => {
  if (searchTimeout) {
    clearTimeout(searchTimeout)
  }
  searchTimeout = setTimeout(() => {
    currentPage.value = 1
    loadCustomers()
  }, 300)
}

// 筛选变化处理
const handleFilterChange = () => {
  currentPage.value = 1
  loadCustomers()
}

// 分页处理
const handlePageChange = (page: number) => {
  currentPage.value = page
  loadCustomers()
}

const handlePageSizeChange = (size: number) => {
  pageSize.value = size
  currentPage.value = 1
  loadCustomers()
}

// 刷新数据
const handleRefresh = () => {
  loadCustomers()
}

// 客户操作处理
const handleViewCustomer = (customer: Customer) => {
  router.push(`/customers/${customer.id}`)
}

const handleEditCustomer = (customer: Customer) => {
  editingCustomer.value = customer
  showCreateModal.value = true
}

const handleDeleteCustomer = (customer: Customer) => {
  deletingCustomer.value = customer
  showDeleteModal.value = true
}

const handleTrackCustomer = (customer: Customer) => {
  router.push(`/customers/${customer.id}#track`)
}

// 表单成功处理
const handleFormSuccess = () => {
  showCreateModal.value = false
  editingCustomer.value = null
  loadCustomers()
  message.success('操作成功')
}

// 确认删除
const confirmDelete = async () => {
  if (!deletingCustomer.value) return
  
  try {
    deleting.value = true
    await customerStore.deleteCustomer(deletingCustomer.value.id)
    message.success('删除成功')
    showDeleteModal.value = false
    deletingCustomer.value = null
    loadCustomers()
  } catch (error: any) {
    message.error(error.message || '删除失败')
  } finally {
    deleting.value = false
  }
}

// 退出登录
const handleLogout = async () => {
  try {
    await authStore.logout()
    message.success('已退出登录')
    router.push('/login')
  } catch (error: any) {
    message.error(error.message || '退出登录失败')
  }
}

// 页面加载
onMounted(() => {
  loadCustomers()
})
</script>

<style scoped>
.customer-list-container {
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
  align-items: center;
}

.header-info h1 {
  margin: 0 0 4px 0;
  font-size: 24px;
  font-weight: 600;
  color: #333;
}

.filter-card {
  margin-bottom: 16px;
}

.filter-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.search-section {
  width: 100%;
}

.search-input {
  width: 100%;
  max-width: 400px;
}

.filter-section {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
}

.filter-controls {
  display: flex;
  gap: 12px;
  align-items: center;
}

.filter-select {
  width: 140px;
  min-width: 120px;
}

.stats-section {
  flex-shrink: 0;
}

.stats-text {
  font-size: 14px;
  white-space: nowrap;
}

.list-card {
  margin-bottom: 16px;
  min-height: 400px;
}

.empty-state {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 300px;
}

.customer-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 16px;
}

.pagination-container {
  display: flex;
  justify-content: center;
  padding: 20px 0;
}

/* 平板响应式 */
@media (max-width: 1024px) {
  .filter-section {
    flex-direction: column;
    align-items: stretch;
    gap: 12px;
  }
  
  .filter-controls {
    justify-content: flex-start;
    flex-wrap: wrap;
  }
  
  .stats-section {
    align-self: center;
  }
}

/* 移动端响应式 */
@media (max-width: 768px) {
  .customer-list-container {
    padding: 16px;
  }
  
  .header-content {
    flex-direction: column;
    align-items: flex-start;
    gap: 16px;
  }
  
  .filter-content {
    gap: 12px;
  }
  
  .search-input {
    max-width: none;
  }
  
  .filter-section {
    flex-direction: column;
    align-items: stretch;
    gap: 12px;
  }
  
  .filter-controls {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
  }
  
  .filter-select {
    width: 100%;
    min-width: unset;
  }
  
  .stats-section {
    text-align: center;
    padding: 8px 0;
    border-top: 1px solid #f0f0f0;
  }
  
  .customer-grid {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 480px) {
  .customer-list-container {
    padding: 12px;
  }
  
  .header-info h1 {
    font-size: 20px;
  }
  
  .filter-content {
    gap: 8px;
  }
  
  .filter-controls {
    grid-template-columns: 1fr;
    gap: 6px;
  }
  
  .stats-text {
    font-size: 13px;
  }
}

/* 删除确认弹窗移动端优化 */
.delete-modal :deep(.n-dialog) {
  border-radius: 12px;
}

.delete-modal :deep(.n-dialog__title) {
  font-size: 18px;
  font-weight: 600;
  padding: 20px 24px 16px 24px;
}

.delete-modal :deep(.n-dialog__content) {
  padding: 0 24px 16px 24px;
  font-size: 16px;
  line-height: 1.5;
}

.delete-modal :deep(.n-dialog__action) {
  padding: 16px 24px 24px 24px;
}

@media (max-width: 768px) {
  .delete-modal {
    margin: 16px;
  }
  
  .delete-modal :deep(.n-dialog) {
    border-radius: 12px;
    max-width: none;
    width: calc(100vw - 32px);
  }

  .delete-modal :deep(.n-dialog__title) {
    padding: 20px 20px 16px 20px;
    font-size: 18px;
  }

  .delete-modal :deep(.n-dialog__content) {
    padding: 0 20px 16px 20px;
    font-size: 16px;
  }

  .delete-modal :deep(.n-dialog__action) {
    padding: 16px 20px 20px 20px;
  }

  /* 移动端按钮优化 */
  .delete-modal :deep(.n-space) {
    width: 100%;
    justify-content: stretch;
  }
  
  .delete-modal :deep(.n-button) {
    flex: 1;
    height: 44px;
    font-size: 16px;
    font-weight: 500;
  }
}

@media (max-width: 480px) {
  .delete-modal {
    margin: 8px;
  }
  
  .delete-modal :deep(.n-dialog) {
    width: calc(100vw - 16px);
    border-radius: 8px;
  }

  .delete-modal :deep(.n-dialog__title) {
    padding: 16px 16px 12px 16px;
    font-size: 17px;
  }

  .delete-modal :deep(.n-dialog__content) {
    padding: 0 16px 12px 16px;
    font-size: 15px;
  }

  .delete-modal :deep(.n-dialog__action) {
    padding: 12px 16px 16px 16px;
  }

  /* 小屏按钮堆叠布局 */
  .delete-modal :deep(.n-space) {
    flex-direction: column;
    gap: 8px !important;
  }
  
  .delete-modal :deep(.n-button) {
    width: 100%;
    height: 48px;
  }
}
</style>
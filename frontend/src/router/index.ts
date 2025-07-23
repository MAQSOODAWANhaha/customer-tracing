import { createRouter, createWebHistory } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const routes = [
  {
    path: '/login',
    name: 'Login',
    component: () => import('@/views/Login.vue'),
    meta: { 
      requiresAuth: false,
      title: '登录'
    }
  },
  {
    path: '/',
    redirect: '/customers'
  },
  {
    path: '/customers',
    name: 'CustomerList',
    component: () => import('@/views/CustomerList.vue'),
    meta: { 
      requiresAuth: true,
      title: '客户管理'
    }
  },
  {
    path: '/customers/:id',
    name: 'CustomerDetail',
    component: () => import('@/views/CustomerDetail.vue'),
    meta: { 
      requiresAuth: true,
      title: '客户详情'
    }
  },
  {
    path: '/:pathMatch(.*)*',
    name: 'NotFound',
    component: () => import('@/views/NotFound.vue'),
    meta: {
      title: '页面不存在'
    }
  }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

// Global navigation guard
router.beforeEach(async (to, _from, next) => {
  const authStore = useAuthStore()

  // Update document title
  if (to.meta.title) {
    document.title = `${to.meta.title} - ${import.meta.env.VITE_APP_TITLE || '客户追踪系统'}`
  }

  // Check if route requires authentication
  if (to.meta.requiresAuth) {
    // Check if user is authenticated
    if (!authStore.isAuthenticated) {
      // Try to restore auth state from localStorage
      const restored = await authStore.initAuth()
      
      if (!restored) {
        // Redirect to login page with return path
        next({
          path: '/login',
          query: { redirect: to.fullPath }
        })
        return
      }
    }
  }
  
  // If authenticated user tries to access login page, redirect to dashboard
  if (to.path === '/login' && authStore.isAuthenticated) {
    next('/customers')
    return
  }
  
  next()
})

// Global after guard
router.afterEach((to, from) => {
  // You can add analytics tracking here
  console.log(`Navigated from ${from.path} to ${to.path}`)
})

export default router
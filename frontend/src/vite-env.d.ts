/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_API_BASE_URL: string
  readonly VITE_APP_TITLE: string
  readonly VITE_TOKEN_STORAGE_KEY: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}
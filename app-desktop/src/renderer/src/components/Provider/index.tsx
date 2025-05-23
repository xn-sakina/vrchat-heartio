import { ConfigProvider } from '@arco-design/web-react'

import zhCN from '@arco-design/web-react/es/locale/zh-CN'
import enUS from '@arco-design/web-react/es/locale/en-US'
import { Toaster } from 'sonner'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'

const hasCN = navigator.language.toLowerCase().includes('zh')

const getLocale = () => {
  if (hasCN) {
    return zhCN
  }
  return enUS
}

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      refetchOnWindowFocus: false,
    },
  },
})

export const Provider = ({ children }: { children: React.ReactNode }) => {
  return (
    <ConfigProvider locale={getLocale()}>
      <QueryClientProvider client={queryClient}>
        {children}
        <Toaster richColors />
      </QueryClientProvider>
    </ConfigProvider>
  )
}

import { createRoot } from 'react-dom/client'

import 'modern-normalize/modern-normalize.css'
import '@mantine/core/styles.css'
import { MantineProvider } from '@mantine/core'
import App from './App'

createRoot(document.getElementById('root')!).render(
  <MantineProvider>
    <App />
  </MantineProvider>,
)

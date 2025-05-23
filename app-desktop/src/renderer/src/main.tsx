import 'modern-normalize/modern-normalize.css'
import '@arco-design/web-react/dist/css/arco.css'

import { createRoot } from 'react-dom/client'
import App from './App'
import { Provider } from './components/Provider'

createRoot(document.getElementById('root')!).render(
  <Provider>
    <App />
  </Provider>,
)

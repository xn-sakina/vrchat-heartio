import { EIpcEvents } from './constants'
import { useEffect, useState } from 'react'
import styles from './App.module.scss'

export default function App() {
  const [loading, setLoading] = useState(true)
  const [errorMsg, setErrorMsg] = useState('')

  const getGlobalConfig = async () => {
    const config = await window.electron.ipcRenderer.invoke(
      EIpcEvents.getGlobalConfig,
    )
    console.log('config: ', config)
  }

  useEffect(() => {
    const init = async () => {
      try {
        throw new Error('test error')
        await getGlobalConfig()
      } catch (err: any) {
        setErrorMsg(err?.message || err?.toString() || 'Unknown error occurred')
      } finally {
        setLoading(false)
      }
    }
    init()
  }, [])

  return <div className={styles.app}></div>
}

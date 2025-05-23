import { useEffect, useState } from 'react'
import styles from './App.module.scss'
import { Menu, Result, Spin } from '@arco-design/web-react'
import { IconLink, IconHeart } from '@arco-design/web-react/icon'
import { toast } from 'sonner'
import { Home } from './pages/Home'
import { Graph } from './pages/Graph'
import { Ipc } from './ipc'

enum EMenu {
  home = 'home',
  graph = 'graph',
}

export default function App() {
  const [loading, setLoading] = useState(true)
  const [menu, setMenu] = useState(EMenu.home)
  const [isFatalError, setIsFatalError] = useState(false)

  const getGlobalConfig = async () => {
    const config = await Ipc.getGlobalConfig()
    console.log('config: ', config)
  }

  useEffect(() => {
    const init = async () => {
      try {
        await getGlobalConfig()
      } catch (err: any) {
        const errorMsg = `获取配置失败，请重启。(${err.message || '未知错误'})`
        toast.error(errorMsg)
        setIsFatalError(true)
      } finally {
        setLoading(false)
      }
    }
    init()
  }, [])

  const isHomeMenu = menu === EMenu.home
  const isGraphMenu = menu === EMenu.graph

  return (
    <div className={styles.app}>
      <Spin className={styles.spin} dot tip="启动中..." loading={loading}>
        <div className={styles.box}>
          {!loading && !isFatalError && (
            <div className={styles.container}>
              <div className={styles.menu}>
                <Menu
                  style={{ width: 200, height: '100%' }}
                  hasCollapseButton
                  selectedKeys={[menu]}
                  onClickMenuItem={(key) => {
                    setMenu(key as EMenu)
                  }}
                >
                  <Menu.Item key={EMenu.home} className={styles.menu_item}>
                    <IconLink />
                    {`连接设备`}
                  </Menu.Item>
                  <Menu.Item key={EMenu.graph} className={styles.menu_item}>
                    <IconHeart />
                    {`数据回溯`}
                  </Menu.Item>
                </Menu>
              </div>
              <div className={styles.content}>
                {isHomeMenu && <Home />}
                {isGraphMenu && <Graph />}
              </div>
            </div>
          )}
          {isFatalError && (
            <div className={styles.fatal_error}>
              <Result status="error" title="启动失败" subTitle="请重启应用" />
            </div>
          )}
        </div>
      </Spin>
    </div>
  )
}

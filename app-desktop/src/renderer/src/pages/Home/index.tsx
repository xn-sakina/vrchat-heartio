import { useGlobalData } from '@renderer/store'
import styles from './index.module.scss'
import {
  IconHeartFill,
  IconLoading,
  IconPlayArrow,
  IconRecordStop,
  IconStop,
  IconSwap,
} from '@arco-design/web-react/icon'
import cx from 'clsx'
import { Alert, Button, Empty, List, Popconfirm } from '@arco-design/web-react'
import { useState } from 'react'
import { Ipc } from '@renderer/ipc'
import { useLatestDeviceList } from '@renderer/hooks/useLatestDeviceList'
import { IHeartRateGetLatestDevice } from '@shared/interface'
import { toast } from 'sonner'

export const Home = () => {
  const isListening = useGlobalData((state) => state.isListening)
  const currentHeartRate = useGlobalData((state) => state.currentHeartRate)

  // searching
  const [isSearching, setIsSearching] = useState(false)
  const [isSendSearchCmdLoading, setSendSearchCmdLoading] = useState(false)

  // searching hook
  const deviceQuery = useLatestDeviceList({
    enabled: isSearching,
  })
  const deviceList = deviceQuery.data || []

  // search click handler
  const onSearch = async () => {
    try {
      setSendSearchCmdLoading(true)

      const res = await Ipc.startSearching()
      if (!res?.success) {
        // stop
        setSendSearchCmdLoading(false)
        return
      }

      // exit loading
      setSendSearchCmdLoading(false)
      // enter searching
      setIsSearching(true)
    } catch {
      setSendSearchCmdLoading(false)
    }
  }
  const onSearchStop = async () => {
    // stop search query
    setIsSearching(false)
  }

  // connect click handler
  const onConnect = async (device: IHeartRateGetLatestDevice) => {}

  // status label: text and icon
  const getStatusText = () => {
    if (isSendSearchCmdLoading) {
      return (
        <div
          className={cx(
            styles.top_value_status_text_base,
            styles.top_value_status_text_searching,
          )}
        >
          {`准备扫描中...`}
        </div>
      )
    }
    if (isSearching) {
      return (
        <div
          className={cx(
            styles.top_value_status_text_base,
            styles.top_value_status_text_searching,
          )}
        >{`正在扫描...`}</div>
      )
    }
    return (
      <div
        className={cx(
          styles.top_value_status_text_base,
          styles.top_value_status_text_not_listening,
        )}
      >{`未连接`}</div>
    )
  }
  const statusText = getStatusText()
  const getStatusIcon = () => {
    if (isSendSearchCmdLoading || isSearching) {
      return (
        <IconLoading
          className={cx(
            styles.top_value_icon_base,
            styles.top_value_icon_searching,
          )}
        />
      )
    }
    return (
      <IconStop
        className={cx(
          styles.top_value_icon_base,
          styles.top_value_icon_not_listening,
        )}
      />
    )
  }
  const statusIcon = getStatusIcon()

  return (
    <div className={styles.box}>
      <div className={styles.container}>
        <div className={styles.top}>
          <div
            className={cx(styles.title_base, styles.top_title)}
          >{`当前心率`}</div>
          <div className={styles.top_value}>
            {isListening ? (
              <>
                <IconHeartFill
                  className={cx(
                    styles.top_value_icon,
                    isListening && styles.top_value_icon_animation,
                  )}
                />
                <div
                  className={cx(
                    styles.top_value_text,
                    isListening && styles.top_value_text_listening,
                  )}
                >
                  {currentHeartRate}
                </div>
                <div className={styles.top_value_unit}>{`BPM`}</div>
              </>
            ) : (
              <div className={styles.top_value_not_listening_box}>
                {statusIcon}
                {statusText}
              </div>
            )}
          </div>
        </div>
        <div className={styles.bottom}>
          <div
            className={cx(styles.title_base, styles.device_title)}
          >{`设备列表`}</div>
          <div className={styles.device_list}>
            {isSearching ? (
              <div className={styles.device_list_result_box}>
                <List
                  style={{ width: '100%' }}
                  className={styles.device_list_component}
                  dataSource={deviceList}
                  noDataElement={
                    <Empty
                      description={'未发现任何设备'}
                      className={styles.device_empty}
                    />
                  }
                  render={(item, index) => {
                    // for label
                    const deviceNameCN = item?.name || '未知设备'
                    const deviceAddrCN = item?.address || '未知地址'
                    // for key
                    const deviceKey =
                      item?.address || `${item?.name || 'unknow'}-${index}`

                    // for toast
                    const toastName = item?.name?.length
                      ? item.name
                      : item?.address?.length
                        ? item.address
                        : ''

                    return (
                      <List.Item key={deviceKey} className={styles.device_line}>
                        <div className={styles.device_item_box}>
                          <div className={styles.device_item_left}>
                            <span
                              className={styles.device_item_name_label}
                            >{`设备`}</span>
                            <span className={styles.device_item_name_value}>
                              {deviceNameCN}
                            </span>
                            <span
                              className={styles.device_item_addr_label}
                            >{`地址`}</span>
                            <span className={styles.device_item_addr_value}>
                              {deviceAddrCN}
                            </span>
                          </div>
                          <div className={styles.device_item_right}>
                            <Popconfirm
                              focusLock
                              title="确认连接此设备？"
                              cancelText="取消"
                              okText="连接"
                              onOk={() => {
                                const msg = !toastName?.length
                                  ? `正在连接设备...`
                                  : `正在连接设备：${toastName}`
                                toast.info(msg)
                              }}
                            >
                              <Button
                                icon={<IconSwap />}
                                type="text"
                                size="mini"
                                onClick={() => {
                                  onConnect(item)
                                }}
                              >
                                {`连接`}
                              </Button>
                            </Popconfirm>
                          </div>
                        </div>
                      </List.Item>
                    )
                  }}
                />
                <Button
                  icon={<IconRecordStop />}
                  type="dashed"
                  status="danger"
                  onClick={() => {
                    onSearchStop()
                  }}
                  className={styles.search_btn}
                >
                  {`停止搜索`}
                </Button>
              </div>
            ) : (
              <div className={styles.device_list_not_searching}>
                <Alert
                  showIcon={false}
                  content={
                    <div className={styles.device_list_not_searching_tips}>
                      <div>{'1、开启电脑蓝牙'} </div>
                      <div>{'2、开启心率设备（手环、手表需开启心率广播）'}</div>
                      <div>{'3、点击开始搜索设备'} </div>
                    </div>
                  }
                />
                <Button
                  loading={isSendSearchCmdLoading}
                  icon={<IconPlayArrow />}
                  type="primary"
                  onClick={() => {
                    onSearch()
                  }}
                  className={styles.search_btn}
                >
                  {isSendSearchCmdLoading ? '搜索中...' : `开始搜索设备`}
                </Button>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  )
}

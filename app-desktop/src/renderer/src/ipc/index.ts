import { EIpcEvents } from '@shared/constants'
import {
  IHeartRateGetLatestDeviceListRes,
  IHeartRateStartSearchingRes,
} from '@shared/interface'

export class Ipc {
  static async getGlobalConfig() {
    const res = await window.electron.ipcRenderer.invoke(
      EIpcEvents.getGlobalConfig,
    )
    console.log('[IPC] getGlobalConfig res: ', res)
    return res
  }

  static async startSearching() {
    const res = await window.electron.ipcRenderer.invoke(
      EIpcEvents.startSearching,
    )
    console.log('[IPC] startSearching res: ', res)
    return res as IHeartRateStartSearchingRes | undefined
  }

  static async stopSearching() {
    const res = await window.electron.ipcRenderer.invoke(
      EIpcEvents.stopSearching,
    )
    console.log('[IPC] stopSearching res: ', res)
    return res
  }

  static async getLatestDeviceList() {
    const res = await window.electron.ipcRenderer.invoke(
      EIpcEvents.getLatestDeviceList,
    )
    console.log('[IPC] getLatestDeviceList res: ', res)
    return res as IHeartRateGetLatestDeviceListRes | undefined
  }
}

import {
  IHeartRateGetLatestDeviceListRes,
  IHeartRateStartSearchingRes,
  IHeartRateStopSearchingRes,
} from '@shared/interface'

export class HeartRate {
  static async startSearching() {
    const res: IHeartRateStartSearchingRes = { success: true }
    return res
  }

  static async stopSearching() {
    const res: IHeartRateStopSearchingRes = { success: true }
    return res
  }

  static async getLatestDeviceList() {
    const res: IHeartRateGetLatestDeviceListRes = [
      {
        name: 'Heart Rate Monitor 1',
        address: '00:11:22:33:44:55',
        hasHeartRateSevice: false,
      },
      {
        name: 'Heart Rate Monitor 2',
        address: '66:77:88:99:AA:BB',
        hasHeartRateSevice: false,
      },
    ]
    return res
  }
}

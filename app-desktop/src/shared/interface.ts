export interface IHeartRateStartSearchingRes {
  success: boolean
}
export interface IHeartRateStopSearchingRes {
  success: boolean
}

export interface IHeartRateGetLatestDevice {
  name?: string
  address?: string
  hasHeartRateSevice?: boolean
}
export type IHeartRateGetLatestDeviceListRes = IHeartRateGetLatestDevice[]

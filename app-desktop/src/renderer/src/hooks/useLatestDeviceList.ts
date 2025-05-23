import { Ipc } from '@renderer/ipc'
import { useQuery } from '@tanstack/react-query'

interface ILatestDeviceListOptions {
  enabled?: boolean
}

export const useLatestDeviceList = (opts: ILatestDeviceListOptions) => {
  return useQuery({
    queryKey: ['useLatestDeviceList'],
    queryFn: async () => {
      const list = await Ipc.getLatestDeviceList()
      return list || []
    },
    refetchInterval: 1000,
    enabled: !!opts?.enabled,
  })
}

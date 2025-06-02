import { SafeAreaProvider } from 'react-native-safe-area-context'
import {
  ActivityIndicator,
  Button,
  StyleSheet,
  Text,
  TextInput,
  View,
} from 'react-native'
import { useEffect, useRef, useState } from 'react'
import { requestPermissions } from '@/utils/ble-permission'
import { toast } from 'sonner-native'
import { Device, BleManager, Subscription } from 'react-native-ble-plx'
import { ScrollView } from 'react-native-gesture-handler'
import { KeyboardAvoidingView } from 'react-native-keyboard-controller'
import { Storage } from '@/utils/storage'
import { toByteArray } from 'base64-js'

const HEART_RATE_SERVICE_UUID = '180d'
const HEART_RATE_MEASUREMENT_CHARACTERISTIC_UUID = '2a37'

interface IDevice {
  name: string
  id: string
  serviceUUIDs: string[]
}

interface IMatchResult {
  match: boolean
  uuid?: string
}

interface IConfig {
  ip?: string
}

const isHeartRateDevice = (uuids: string[]) => {
  if (!uuids?.length) {
    return {
      match: false,
    } as IMatchResult
  }
  let targetUUID: string | undefined
  const hasTargetUUID = uuids.some((id) => {
    if (!id?.length) {
      return
    }
    let lowerCaseId = id.toLowerCase()
    if (lowerCaseId.includes('-')) {
      lowerCaseId = lowerCaseId.split('-')?.[0]
    }
    if (!lowerCaseId?.length) {
      return false
    }
    const isMatch = lowerCaseId.endsWith(HEART_RATE_SERVICE_UUID)
    if (isMatch) {
      targetUUID = id
    }
    return isMatch
  })
  if (hasTargetUUID) {
    return {
      match: true,
      uuid: targetUUID,
    } as IMatchResult
  }
  return {
    match: false,
  } as IMatchResult
}

export default function Home() {
  const [hasAuth, setHasAuth] = useState<boolean | undefined>()
  const [allDevices, setAllDevices] = useState<IDevice[]>([])
  const [isScanning, setIsScanning] = useState(false)

  // target info
  const [device, setDevice] = useState<Device | undefined>()
  const [subUUID, setSubUUID] = useState<string | undefined>()
  const [subscription, setSubscription] = useState<Subscription | undefined>()

  // bpm
  const [currentBPM, setCurrentBPM] = useState('')

  // input
  const [ipConfig, setIpConfig] = useState('')

  // api error
  const apiErrorCountRef = useRef(0)

  // config
  const loadConfig = async () => {
    const config = (await Storage.loadData()) as IConfig | undefined
    if (!config?.ip?.length) {
      return
    }
    setIpConfig(config.ip)
  }
  const saveConfig = async (config: IConfig) => {
    await Storage.saveData(config)
  }

  useEffect(() => {
    const func = async () => {
      // load config
      await loadConfig()
      // req ble permission
      const isAllow = await requestPermissions()
      if (isAllow) {
        setHasAuth(true)
      } else {
        setHasAuth(false)

        // toast
        toast.error(`Bluetooth permission is not allowed`, {
          duration: 10 * 1e3,
        })
        return
      }
    }
    func()
  }, [])

  // api error count
  const getApiErrorCount = () => {
    return apiErrorCountRef.current
  }
  const plusApiErrorCount = () => {
    apiErrorCountRef.current += 1
  }
  const resetApiErrorCount = () => {
    apiErrorCountRef.current = 0
  }

  const sendToServer = async (data: { bpm: number }) => {
    if (!ipConfig?.length || !ipConfig?.trim()?.length) {
      toast.error(`Please input your PC internal network IP`, {
        duration: 3 * 1e3,
      })
      // stop sub
      if (subscription) {
        subscription.remove()
        setSubscription(undefined)
      }
      return
    }
    if (!data?.bpm || data.bpm <= 0) {
      toast.error(`Invalid BPM value: ${data.bpm}`, {
        duration: 2 * 1e3,
      })
      return
    }
    if (!device) {
      toast.error(`No device connected`, {
        duration: 2 * 1e3,
      })
      return
    }
    // limit
    const apiErrorCount = getApiErrorCount()
    if (apiErrorCount > 20) {
      return
    }

    const trimmedIP = ipConfig.trim()
    const url = `http://${trimmedIP}:2333/heart?bpm=${data.bpm}`
    // send
    try {
      const response = await fetch(url, {
        method: 'GET',
      })
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`)
      }
      // ok
    } catch (error: any) {
      toast.error(`Error sending data to server: ${error.message}`, {
        duration: 1 * 1e3,
      })

      // plus
      plusApiErrorCount()

      // limit
      const errorCount = getApiErrorCount()
      if (errorCount > 20) {
        toast.error(
          `API error count exceeded, please check your server or network`,
          {
            duration: 5 * 1e3,
          },
        )
        // cleanup
        cleanup()
      }
    }
  }

  const subDevice = async (newDevice: Device, targetUUID: string) => {
    if (device) {
      return
    }
    if (!newDevice || !targetUUID) {
      return
    }
    // set device
    setDevice(newDevice)
    // set sub UUID
    setSubUUID(targetUUID)

    await newDevice.connect()
    await newDevice.discoverAllServicesAndCharacteristics()

    // get characteristic
    const getHeartRateChar = async () => {
      const allServices = await newDevice.services()
      if (!allServices || !allServices.length) {
        toast.error(
          `No services found for device ${newDevice.name || 'Unknown'}`,
          {
            duration: 5 * 1e3,
          },
        )
        return
      }
      const targetService = allServices.find((service) => {
        const id = service.uuid
        if (!id?.length) {
          return false
        }
        const isMatch = id === targetUUID
        return isMatch
      })
      if (!targetService) {
        toast.error(
          `No target service found for device ${newDevice.name || 'Unknown'}`,
          {
            duration: 5 * 1e3,
          },
        )
        return
      }
      const allChars = await targetService.characteristics()
      if (!allChars?.length) {
        toast.error(`No characteristics found for service ${targetUUID}`, {
          duration: 5 * 1e3,
        })
        return
      }
      const targetChar = allChars.find((char) => {
        if (!char) {
          return false
        }
        let lowerCaseUUID = char.uuid.toLowerCase()
        if (!lowerCaseUUID?.length) {
          return false
        }
        if (lowerCaseUUID.includes('-')) {
          lowerCaseUUID = lowerCaseUUID.split('-')?.[0]
        }
        if (!lowerCaseUUID?.length) {
          return false
        }
        const isMatch = lowerCaseUUID.endsWith(
          HEART_RATE_MEASUREMENT_CHARACTERISTIC_UUID,
        )
        return isMatch
      })
      if (!targetChar) {
        toast.error(
          `No target characteristic found for service ${targetUUID}`,
          {
            duration: 5 * 1e3,
          },
        )
        return
      }
      return targetChar
    }
    toast.info(`Find heart rate characteristic...`, {
      duration: 3 * 1e3,
    })
    const heartRateChar = await getHeartRateChar()
    if (!heartRateChar) {
      toast.error(`No heart rate characteristic found`, {
        duration: 5 * 1e3,
      })
      return
    }

    // subscribe to characteristic
    const sub = heartRateChar.monitor((error, characteristic) => {
      if (error) {
        toast.error(`Error monitoring characteristic: ${error.message}`, {
          duration: 5 * 1e3,
        })
        return
      }
      if (!characteristic?.value) {
        toast.error(`No value received from characteristic`, {
          duration: 3 * 1e3,
        })
        return
      }

      // get bpm
      let bpm = 0

      try {
        const bytes = toByteArray(characteristic.value)
        const flag = bytes[0]

        if ((flag & 0x01) === 0) {
          bpm = bytes[1] // UINT8
        } else {
          bpm = bytes[1] + (bytes[2] << 8) // uint16, Little Endian
        }

        if (bpm === undefined) {
          toast.error(`No heart rate value found`, {
            duration: 2 * 1e3,
          })
          return
        }
      } catch {
        toast.error(`Error parsing heart rate value`, {
          duration: 2 * 1e3,
        })
        return
      }

      // update current BPM
      setCurrentBPM(`${bpm}`)

      // send to server
      sendToServer({
        bpm,
      })
    })
    // set sub
    setSubscription(sub)

    toast.success(`Subscribed to heart rate characteristic`, {
      duration: 3 * 1e3,
    })
  }

  const cleanup = () => {
    // clear all devices
    setAllDevices([])

    // cleanup device and subscription
    setDevice(undefined)
    setSubUUID(undefined)
    // stop sub
    if (subscription) {
      subscription.remove()
    }
    setSubscription(undefined)

    // clear current BPM
    setCurrentBPM('')

    // reset api error count
    resetApiErrorCount()
  }

  const scanDevices = async () => {
    // clear
    cleanup()

    // start scanning
    setIsScanning(true)

    let timeoutTimer: any = setTimeout(() => {
      setIsScanning(false)
      // clear
      if (timeoutTimer) {
        clearTimeout(timeoutTimer)
        timeoutTimer = null
      }
      toast.warning('Scan timed out, Stop', {
        duration: 3 * 1e3,
      })
    }, 30 * 1e3)

    try {
      const manager = new BleManager()
      toast.info('Scanning for devices...', {
        duration: 3 * 1e3,
      })

      let isFound = false

      const getDevices = async () => {
        manager.startDeviceScan(null, null, async (error, device) => {
          if (error) {
            toast.error(`Error scanning for devices: ${error.message}`, {
              duration: 5 * 1e3,
            })
            return
          }

          if (isFound) {
            // already found a device, stop scanning
            return
          }

          if (!device) {
            return
          }

          const uniqID = device.id
          const deviceName = device?.localName || `Unknown Device`
          const deviceId = device?.id
          const serviceUUIDs = device.serviceUUIDs || []

          const { match: isTargetDevice, uuid: targetUUID } =
            isHeartRateDevice(serviceUUIDs)
          if (isTargetDevice) {
            // stop
            toast.success(`Found Heart Rate device: ${deviceName}`, {
              duration: 3 * 1e3,
            })
            // sub
            isFound = true
            // stop scanning
            await manager.stopDeviceScan()
            // update state
            setIsScanning(false)
            clearTimeout(timeoutTimer)
            timeoutTimer = null
            // subscribe to device
            await subDevice(device, targetUUID!)
            return
          }

          // add to allDevices
          setAllDevices((prev) => {
            const isUniq = prev.some((d) => d.id === uniqID)
            if (isUniq) {
              return prev
            }
            const newDevice: IDevice = {
              name: deviceName,
              id: deviceId,
              serviceUUIDs: serviceUUIDs,
            }
            return [...prev, newDevice]
          })
        })
      }
      await getDevices()
    } finally {
    }
  }

  if (hasAuth === undefined) {
    return (
      <SafeAreaProvider>
        <View style={styles.container}>
          <Text style={styles.text}>Loading...</Text>
        </View>
      </SafeAreaProvider>
    )
  }

  if (hasAuth === false) {
    return (
      <SafeAreaProvider>
        <View style={styles.container}>
          <Text style={styles.auth_none}>
            Please allow bluetooth permission in settings and restart the app.
          </Text>
        </View>
      </SafeAreaProvider>
    )
  }

  const isSubscribed = !!device
  const cannotScan = isSubscribed || !hasAuth || isScanning || !ipConfig?.length

  return (
    <SafeAreaProvider>
      <KeyboardAvoidingView
        style={{ flex: 1 }}
        behavior="padding"
        keyboardVerticalOffset={10}
      >
        <ScrollView style={{ flex: 1 }}>
          <View style={styles.container}>
            <View style={styles.box}>
              <View style={styles.list}>
                {!isSubscribed ? (
                  <>
                    <Text style={styles.title}>{`ALL DEVICES:`}</Text>
                    {!!allDevices?.length ? (
                      <ScrollView style={styles.scroll_list}>
                        {allDevices.map((d) => (
                          <View key={d.id} style={{ paddingBottom: 10 }}>
                            <Text style={styles.text}>{`Name: ${d.name}`}</Text>
                            <Text style={styles.text}>{`ID: ${d.id}`}</Text>
                            <Text
                              style={styles.text}
                            >{`Service UUIDs: ${d.serviceUUIDs.join(
                              ', ',
                            )}`}</Text>
                          </View>
                        ))}
                      </ScrollView>
                    ) : isScanning ? (
                      <ActivityIndicator />
                    ) : (
                      <Text style={styles.text}>No devices found</Text>
                    )}
                  </>
                ) : (
                  <>
                    <Text
                      style={styles.title}
                    >{`CURRENT CONNECTED DEVICE:`}</Text>
                    <Text style={styles.text}>{`Name: ${
                      device?.name || 'Unknown'
                    }`}</Text>
                    <Text style={styles.text}>{`ID: ${
                      device?.id || 'Unknown'
                    }`}</Text>
                    <Text style={styles.text}>{`Service UUID: ${
                      subUUID || 'Unknown'
                    }`}</Text>
                    {currentBPM ? (
                      <Text style={{ color: 'green' }}>{`BPM: ${
                        currentBPM || 0
                      }`}</Text>
                    ) : (
                      <Text style={styles.text}>{`BPM Loading...`}</Text>
                    )}
                  </>
                )}
              </View>
              <View style={styles.config}>
                <Text style={styles.title}>{`PC Internal Network IP:`}</Text>
                <TextInput
                  value={ipConfig}
                  onChangeText={(text) => setIpConfig(text)}
                  style={styles.input}
                  placeholder="192.168.1.120"
                />
              </View>
              <View style={styles.btn}>
                <Button
                  title="Click to Scan"
                  disabled={cannotScan}
                  color={cannotScan ? '#ccc' : '#007bff'}
                  onPress={() => {
                    if (cannotScan) {
                      return
                    }
                    scanDevices()
                    // save config
                    saveConfig({ ip: ipConfig.trim() })
                  }}
                />
              </View>
            </View>
          </View>
        </ScrollView>
      </KeyboardAvoidingView>
    </SafeAreaProvider>
  )
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    backgroundColor: '#fff',
  },
  auth_none: {
    color: 'red',
    fontSize: 16,
    textAlign: 'center',
    paddingHorizontal: 20,
    fontWeight: '500',
  },
  box: {
    flex: 1,
    width: '100%',
    padding: 10,
  },
  list: {
    padding: 5,
    paddingTop: 50,
    height: 500,
  },
  config: {
    padding: 5,
  },
  title: {
    paddingBottom: 10,
    fontSize: 16,
    fontWeight: '500',
    color: '#000',
  },
  input: {
    borderWidth: 1,
    borderColor: '#ccc',
    borderRadius: 5,
    paddingHorizontal: 10,
    paddingVertical: 5,
    color: '#000',
  },
  text: {
    color: '#000',
  },
  scroll_list: {
    borderWidth: 1,
    borderColor: '#ccc',
    borderRadius: 5,
    paddingHorizontal: 10,
    paddingVertical: 5,
    color: '#000',
  },
  btn: {
    width: '100%',
    paddingHorizontal: 5,
    paddingVertical: 10,
  },
})

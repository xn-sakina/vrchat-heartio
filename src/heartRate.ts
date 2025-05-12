import { isNumber, sample } from 'lodash'
import noble, { Peripheral } from '@stoprocent/noble'
import { sendOscMessage } from './osc'
import path from 'path'
import fs from 'fs'
import dayjs from 'dayjs'
import Database, { type Database as DatabaseIns } from 'better-sqlite3'
import { createLogger } from './utils'
import { spawn } from 'child_process'

enum EHeartLevel {
  low = 'low',
  normal = 'normal',
  high = 'high',
  max = 'max',
  ultra = 'ultra',
  extreme = 'extreme',
}

const BPM_PLACEHOLDER = '{{bpm}}'

const HEART_LEVEL_LABEL: Record<EHeartLevel, string | string[]> = {
  [EHeartLevel.low]: `♡ ${BPM_PLACEHOLDER}`,
  [EHeartLevel.normal]: `❤️ ${BPM_PLACEHOLDER}`,
  [EHeartLevel.high]: `💕 ${BPM_PLACEHOLDER} 💕`,
  [EHeartLevel.max]: `❤️💕 ${BPM_PLACEHOLDER} 💕❤️`,
  [EHeartLevel.ultra]: [
    `❤️❤️❤️ ${BPM_PLACEHOLDER} ❤️❤️❤️`,
    `💕💕💕 ${BPM_PLACEHOLDER} 💕💕💕`,
  ],
  [EHeartLevel.extreme]: [
    `❤️❤️❤️❤️ ${BPM_PLACEHOLDER} ❤️❤️❤️❤️`,
    `💕💕💕💕 ${BPM_PLACEHOLDER} 💕💕💕💕`,
    `LOVE ❤️ ${BPM_PLACEHOLDER} ❤️ LOVE`,
  ],
} as const

const HEART_RATE_SERVICE_UUID = '180d'
const HEART_RATE_MEASUREMENT_CHAR_UUID = '2a37'

const logger = createLogger('HeartRate')

const CACHE_DIR = path.join(__dirname, '../cache')
if (!fs.existsSync(CACHE_DIR)) {
  // create cache directory
  fs.mkdirSync(CACHE_DIR, { recursive: true })
  logger.info('Cache directory created')
}

export class HeartRate {
  private device: Peripheral | undefined
  private prevSendTime = 0

  // for memory leak detection
  private startTimeMemory = 0
  private startTime = 0

  // for db
  private db: DatabaseIns | undefined

  // for timer
  private timerList: NodeJS.Timeout[] = []

  // for timeout checker
  private prevReceiveTime = 0
  private killCaffeinate: (() => void) | undefined

  private async initDataDB() {
    const dbPath = path.join(CACHE_DIR, 'data.sqlite')
    if (this.db) {
      return
    }
    // create db
    const db = new Database(dbPath)
    // create table
    db.exec(`
      CREATE TABLE IF NOT EXISTS heart_rate (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        bpm INTEGER NOT NULL,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP
      )
    `)
    // create index
    db.exec(`
      CREATE INDEX IF NOT EXISTS idx_heart_rate_created_at ON heart_rate (created_at)
    `)
    this.db = db
  }

  private async insertHeartRate(bpm: number) {
    if (!this.db) {
      return
    }
    // insert heart rate
    const stmt = this.db.prepare('INSERT INTO heart_rate (bpm) VALUES (?)')
    const info = stmt.run(bpm)
    const isError = info.changes !== 1
    if (isError) {
      logger.error('Error inserting heart rate:', info)
      return
    }
  }

  private static getHeartRateString(bpm: number) {
    const isValid = isNumber(bpm) && !Number.isNaN(bpm) && bpm > 0
    if (!isValid) {
      logger.error('Invalid heart rate value:', bpm)
      return
    }
    const getPlaceholder = (level: EHeartLevel) => {
      const placeholder = HEART_LEVEL_LABEL[level]
      if (!placeholder) {
        throw new Error(`Invalid heart level: ${level}`)
      }
      if (Array.isArray(placeholder)) {
        const randomItem = sample(placeholder)
        return randomItem!
      }
      return placeholder
    }
    let placeholder: string
    if (bpm < 70) {
      placeholder = getPlaceholder(EHeartLevel.low)
    } else if (bpm < 80) {
      placeholder = getPlaceholder(EHeartLevel.normal)
    } else if (bpm < 100) {
      placeholder = getPlaceholder(EHeartLevel.high)
    } else if (bpm < 130) {
      placeholder = getPlaceholder(EHeartLevel.max)
    } else if (bpm < 150) {
      placeholder = getPlaceholder(EHeartLevel.ultra)
    } else {
      // >= 150
      placeholder = getPlaceholder(EHeartLevel.extreme)
    }
    const text = placeholder.replace(BPM_PLACEHOLDER, bpm.toString())
    return text
  }

  private async exit(errorMsg?: string) {
    if (errorMsg?.length) {
      logger.error('Error:', errorMsg)
    }
    logger.info('Exiting...')
    // clear all timers
    this.clearAllTimers()
    // kill caffeinate
    if (this.killCaffeinate) {
      this.killCaffeinate()
    }
    // stop scanning
    try {
      await noble.stopScanningAsync()
    } catch {
      logger.error('Error stopping scanning')
    }
    // disconnect
    if (this.device) {
      try {
        await this.device.disconnectAsync()
        logger.info(
          'Disconnected from device:',
          this.device.advertisement.localName,
        )
      } catch {
        logger.error('Error disconnecting from device')
      }
    }
    // close db
    if (this.db) {
      try {
        this.db.close()
        logger.info('Database closed')
      } catch {
        logger.error('Error closing database')
      }
    }
    if (errorMsg?.length) {
      process.exit(1)
    } else {
      process.exit(0)
    }
  }

  private async startListenHeartRate() {
    const localDeviceName = process.env.HEART_RATE_DEVICE_NAME
    if (!localDeviceName?.length) {
      // exit with error
      await this.exit('HEART_RATE_DEVICE_NAME is not set')
      return
    } else {
      logger.info(`Looking for device: ${localDeviceName}`)
    }

    // Discover peripherals as an async generator
    let device: Peripheral | undefined
    try {
      logger.info('Starting discovery...')
      // Wait for Adapter poweredOn state
      await noble.waitForPoweredOnAsync()
      // Start scanning first
      await noble.startScanningAsync()

      let startDiscoveryTime = Date.now()

      // Use the async generator with proper boundaries
      for await (const peripheral of noble.discoverAsync()) {
        const duration = Date.now() - startDiscoveryTime
        const timeoutTime = 10 * 1e3
        if (duration > timeoutTime) {
          // exit
          await this.exit(
            `Discovery timeout (${timeoutTime / 1e3}s), no device found`,
          )
          return
        }

        if (peripheral.advertisement.localName === localDeviceName) {
          // connect
          await peripheral.connectAsync()
          device = peripheral
          logger.info(
            'Connected to device:',
            peripheral.advertisement.localName,
          )
          break
        }
      }

      // Clean up after discovery
      await noble.stopScanningAsync()
    } catch (error) {
      logger.error('Discovery error:', error)
      await this.exit('Discovery error')
      return
    }

    if (!device) {
      await this.exit('Device not found')
      return
    }

    // find characteristic
    const { characteristics } =
      await device.discoverSomeServicesAndCharacteristicsAsync(
        [HEART_RATE_SERVICE_UUID],
        [HEART_RATE_MEASUREMENT_CHAR_UUID],
      )
    const hrChar = characteristics[0]

    await hrChar.subscribeAsync()
    logger.info('Subscribed to heart rate characteristic:', hrChar.uuid)

    hrChar.on('data', (data) => {
      // The heart rate data first byte is flags, the second byte is heart rate
      const flags = data.readUInt8(0)
      let heartRate

      if (flags & 0x01) {
        heartRate = data.readUInt16LE(1) // UInt16
      } else {
        heartRate = data.readUInt8(1) // UInt8
      }

      // update receive time
      this.prevReceiveTime = Date.now()

      // Print the heart rate value
      logger.debug('Heart Rate:', heartRate)

      // save to db
      this.insertHeartRate(heartRate).catch((error) => {
        logger.error('Error inserting heart rate:', error)
      })

      // send osc message
      this.sendToOSC(heartRate).catch((error) => {
        logger.error('Error sending OSC message:', error)
      })
    })
  }

  private async sendToOSC(heartRate: number) {
    const text = HeartRate.getHeartRateString(heartRate)
    if (!text) {
      logger.error('Invalid heart rate value:', heartRate)
      return
    }
    const now = Date.now()
    if (now - this.prevSendTime < 1.5 * 1e3) {
      logger.debug('Too fast, skipping send')
      return
    } else {
      const gap = (now - this.prevSendTime) / 1e3
      logger.debug(`Send gap: ${gap}s`)
      // update prevSendTime
      this.prevSendTime = now
    }
    await sendOscMessage(text)
  }

  private async detectMemoryLeak() {
    const memoryUsage = process.memoryUsage()
    const memoryUsageInMB = Math.round(memoryUsage.heapUsed / 1024 / 1024)

    // first time
    if (!this.startTimeMemory) {
      this.startTimeMemory = memoryUsageInMB
      return false
    }

    const maybeMemoryLeak = memoryUsageInMB - this.startTimeMemory > 50
    if (maybeMemoryLeak) {
      const duration = (Date.now() - this.startTime) / 1e3
      logger.warn(
        `Memory leak detected: ${memoryUsageInMB}MB, system uptime: ${duration}s`,
      )

      // write to file
      const memoryLeakDir = path.join(CACHE_DIR, 'memory_leak')
      if (!fs.existsSync(memoryLeakDir)) {
        fs.mkdirSync(memoryLeakDir, { recursive: true })
      }
      const timeLabel = dayjs().format('YYYY-MM-DD_HH-mm-ss')
      const fileName = path.join(memoryLeakDir, `memory_leak_${timeLabel}.json`)
      // write
      const data = {
        time: timeLabel,
        memoryUsageInMB,
        systemUptime: `${duration}s`,
      }
      fs.writeFileSync(fileName, JSON.stringify(data, null, 2))

      return true
    }
    return false
  }

  private async initTimeoutChecker() {
    const timeoutTime = 20 * 1e3 // 20 seconds
    const timer = setInterval(async () => {
      logger.debug('Checking for timeout...')

      if (!this.prevReceiveTime) {
        // pass
        return
      }

      const now = Date.now()
      const duration = now - this.prevReceiveTime
      const isTimeout = duration > timeoutTime
      if (isTimeout) {
        logger.warn(`Timeout detected: ${duration / 1e3}s, no data received`)
        // exit process
        await this.exit('Timeout detected, no data received')
      }
    }, 5 * 1e3)
    // push
    this.timerList.push(timer)
  }

  private async startMemoryLeakDetector() {
    this.startTime = Date.now()

    const timer = setInterval(async () => {
      await this.detectMemoryLeak()
    }, 5 * 1e3)
    // push
    this.timerList.push(timer)
  }

  private clearAllTimers() {
    if (this.timerList.length) {
      this.timerList.forEach((timer) => {
        clearInterval(timer)
      })
      this.timerList = []
    }
  }

  private async listenExitSignal() {
    // listen for exit
    process.on('SIGINT', async () => {
      await this.exit()
    })
    process.on('SIGTERM', async () => {
      await this.exit()
    })
  }

  private async keepSystemAwake() {
    const isMacOs = process.platform === 'darwin'
    if (!isMacOs) {
      return
    }
    // run caffeinate command
    const caffeinate = spawn('caffeinate', ['-d'])

    logger.info('Caffeinate started')

    const killCaffeinate = () => {
      caffeinate.kill()
      logger.info('Caffeinate killed')
    }
    this.killCaffeinate = killCaffeinate
    process.on('SIGINT', () => {
      killCaffeinate()
    })
    process.on('SIGTERM', () => {
      killCaffeinate()
    })
  }

  async start() {
    // listen for exit signal
    await this.listenExitSignal()

    // init db
    await this.initDataDB()

    // start memory leak detector
    await this.startMemoryLeakDetector()

    // start heart rate
    await this.startListenHeartRate()

    // keep system awake
    await this.keepSystemAwake()

    // init timeout checker
    await this.initTimeoutChecker()
  }
}

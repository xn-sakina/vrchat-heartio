import { isNumber } from 'lodash'
import noble, { Peripheral } from '@stoprocent/noble'
import { sendOscMessage } from './osc'
import path from 'path'
import fs from 'fs'
import dayjs from 'dayjs'
import Database, { type Database as DatabaseIns } from 'better-sqlite3'
import { createLogger } from './utils'

enum EHeartLevel {
  normal = 'normal',
  high = 'high',
  super_high = 'super_high',
  full = 'full',
}

const HEART_LEVEL_LABEL: Record<EHeartLevel, string> = {
  [EHeartLevel.normal]: '♡ {{bpm}}',
  [EHeartLevel.high]: '❤️ {{bpm}}',
  [EHeartLevel.super_high]: '💕 {{bpm}} 💕',
  [EHeartLevel.full]: '❤️💕 {{bpm}} 💕❤️',
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
    let placeholder: string
    if (bpm < 70) {
      placeholder = HEART_LEVEL_LABEL[EHeartLevel.normal]
    } else if (bpm < 80) {
      placeholder = HEART_LEVEL_LABEL[EHeartLevel.high]
    } else if (bpm < 90) {
      placeholder = HEART_LEVEL_LABEL[EHeartLevel.super_high]
    } else {
      placeholder = HEART_LEVEL_LABEL[EHeartLevel.full]
    }
    const text = placeholder.replace('{{bpm}}', bpm.toString())
    return text
  }

  private async exit(errorMsg?: string) {
    if (errorMsg?.length) {
      logger.error('Error:', errorMsg)
    }
    logger.info('Exiting...')
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
      // update prevSendTime
      this.prevSendTime = now
      const gap = (now - this.prevSendTime) / 1e3
      logger.debug(`Send gap: ${gap}s`)
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

  private async startMemoryLeakDetector() {
    this.startTime = Date.now()

    setInterval(async () => {
      await this.detectMemoryLeak()
    }, 5 * 1e3)
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

  async start() {
    // listen for exit signal
    await this.listenExitSignal()

    // init db
    await this.initDataDB()

    // start memory leak detector
    await this.startMemoryLeakDetector()

    // start heart rate
    await this.startListenHeartRate()
  }
}

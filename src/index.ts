import './polyfill'

import { EventEmitter } from 'events'
import { config } from 'dotenv'
import { HeartRate } from './heartRate'
import { createConsola } from 'consola'

EventEmitter.defaultMaxListeners = 0

const logger = createConsola({
  defaults: {
    tag: 'App',
  },
  formatOptions: {
    date: true,
  },
})

const loadEnv = () => {
  config()
  logger.info('Environment variables loaded')
}

const run = async () => {
  // load env
  loadEnv()

  // start heart rate
  const heartRate = new HeartRate()
  await heartRate.start()
}

run()

import './setup'

import { EventEmitter } from 'events'
import { HeartRate } from './heartRate'

// prevent event listener warning
EventEmitter.defaultMaxListeners = 0

const run = async () => {
  // start heart rate
  const heartRate = new HeartRate()
  await heartRate.start()
}

run()

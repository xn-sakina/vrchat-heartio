import './polyfill'

import { config } from 'dotenv'
import { createLogger } from './utils'

const logger = createLogger('Setup')

const loadEnv = () => {
  config()
  logger.info('Environment variables loaded')
}

loadEnv()

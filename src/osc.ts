import { Client } from 'node-osc'
import { createLogger } from './utils'

let client: Client | null = null

const logger = createLogger('OSC')

const getEnvConfig = () => {
  let finalHost = '0.0.0.0'
  if (process.env.OSC_HOST) {
    finalHost = process.env.OSC_HOST
  }
  let finalPort = 9000
  if (process.env.OSC_PORT) {
    finalPort = parseInt(process.env.OSC_PORT, 10)
  }
  logger.info(`OSC host: ${finalHost}:${finalPort}`)
  return {
    host: finalHost,
    port: finalPort,
  }
}

const MESSAGE_MAX_LENGTH = 144
const MESSAGE_PATH = '/chatbox/input'

export const sendOscMessage = async (text: string) => {
  if (!client) {
    // new client
    const { host, port } = getEnvConfig()
    client = new Client(host, port)
    logger.success(`Connected to OSC server`)
  }
  // length check
  if (text.length > MESSAGE_MAX_LENGTH) {
    logger.error(
      `Length over ${MESSAGE_MAX_LENGTH} characters, please shorten the message.`,
    )
    return
  }
  const { resolve, promise } = Promise.withResolvers<void>()
  // send
  client.send(
    MESSAGE_PATH,
    text,
    true, // immediate send
    false, // disable SFX
    (err) => {
      if (err) {
        logger.error(`Error sending OSC message: ${err}`)
      } else {
        logger.success(`Sent OSC message: ${text}`)
      }
      resolve()
    },
  )
  return promise
}

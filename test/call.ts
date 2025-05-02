import '../src/setup'

import { sendOscMessage } from '../src/osc'

const run = async () => {
  await sendOscMessage('💕 60 💕')
}

run()

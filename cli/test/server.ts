import { Server } from 'node-osc'

const run = async () => {
  var oscServer = new Server(9000, '0.0.0.0', () => {
    console.log('OSC Server is listening')
  })

  oscServer.on('message', function (msg) {
    console.log(`Message: ${msg}`)
    oscServer.close()
  })
}

run()

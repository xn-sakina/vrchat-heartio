import { app, shell, BrowserWindow, ipcMain } from 'electron'
import { join } from 'path'
import { electronApp, optimizer, is } from '@electron-toolkit/utils'
import icon from '../../resources/icon.png?asset'
import { createLogger } from './utils'
import Store from 'electron-store'
import { HeartRate } from './heartRate'

const logger = createLogger('main')
const store = new Store()

enum EIpcEvents {
  // config
  getGlobalConfig = 'getGlobalConfig',
  setGlobalConfig = 'setGlobalConfig',

  // start searching
  startSearching = 'startSearching',
  stopSearching = 'stopSearching',

  // get latest device list
  getLatestDeviceList = 'getLatestDeviceList',
}

class Storage {
  static keys = {
    globalConfig: 'globalConfig',
  }

  static getGlobalConfig() {
    const config = store.get(Storage.keys.globalConfig) as string | undefined
    return config
  }

  static setGlobalConfig(config: string) {
    store.set(Storage.keys.globalConfig, config)
  }
}

function createWindow(): void {
  // Create the browser window.
  const mainWindow = new BrowserWindow({
    width: 900,
    height: 670,
    show: false,
    autoHideMenuBar: true,
    ...(process.platform === 'linux' ? { icon } : {}),
    webPreferences: {
      preload: join(__dirname, '../preload/index.js'),
      sandbox: false,
    },
  })

  mainWindow.on('ready-to-show', () => {
    mainWindow.show()
  })

  mainWindow.webContents.setWindowOpenHandler((details) => {
    shell.openExternal(details.url)
    return { action: 'deny' }
  })

  // HMR for renderer base on electron-vite cli.
  // Load the remote URL for development or the local html file for production.
  if (is.dev && process.env['ELECTRON_RENDERER_URL']) {
    mainWindow.loadURL(process.env['ELECTRON_RENDERER_URL'])
  } else {
    mainWindow.loadFile(join(__dirname, '../renderer/index.html'))
  }
}

const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms))

const initIPC = () => {
  // config
  ipcMain.handle(EIpcEvents.getGlobalConfig, () => {
    const globalConfig = Storage.getGlobalConfig()
    logger.log('[CALL] globalConfig', globalConfig)
    return globalConfig
  })
  ipcMain.handle(EIpcEvents.setGlobalConfig, (_event, config) => {
    Storage.setGlobalConfig(config)
    logger.log('[CALL] setGlobalConfig', config)
  })

  // start searching
  ipcMain.handle(EIpcEvents.startSearching, async () => {
    await sleep(500)
    logger.log('[CALL] startSearching')
    return HeartRate.startSearching()
  })
  ipcMain.handle(EIpcEvents.stopSearching, async () => {
    await sleep(500)
    logger.log('[CALL] stopSearching')
    return HeartRate.stopSearching()
  })

  // get latest device list
  ipcMain.handle(EIpcEvents.getLatestDeviceList, async () => {
    await sleep(500)
    logger.log('[CALL] getLatestDeviceList')
    return HeartRate.getLatestDeviceList()
  })
}

// This method will be called when Electron has finished
// initialization and is ready to create browser windows.
// Some APIs can only be used after this event occurs.
app.whenReady().then(() => {
  // Set app user model id for windows
  electronApp.setAppUserModelId('org.kanamio.da.heartio')

  // Default open or close DevTools by F12 in development
  // and ignore CommandOrControl + R in production.
  // see https://github.com/alex8088/electron-toolkit/tree/master/packages/utils
  app.on('browser-window-created', (_, window) => {
    optimizer.watchWindowShortcuts(window)
  })

  initIPC()

  createWindow()

  app.on('activate', function () {
    // On macOS it's common to re-create a window in the app when the
    // dock icon is clicked and there are no other windows open.
    if (BrowserWindow.getAllWindows().length === 0) createWindow()
  })
})

// Quit when all windows are closed, except on macOS. There, it's common
// for applications and their menu bar to stay active until the user quits
// explicitly with Cmd + Q.
app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit()
  }
})

// In this file you can include the rest of your app's specific main process
// code. You can also put them in separate files and require them here.

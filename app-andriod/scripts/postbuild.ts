import 'zx/globals'

const run = async () => {
  const outDir = path.join(
    __dirname,
    '../android/app/build/outputs/apk/release',
  )

  const apkFile = await globby([path.join(outDir, '*.apk')], {
    absolute: true,
  })

  if (!apkFile?.length) {
    throw new Error(
      `No APK file found in ${outDir}. Please ensure the build was successful.`,
    )
  }

  const appJsonPath = path.join(__dirname, '../app.json')
  const appJson = JSON.parse(await fs.readFile(appJsonPath, 'utf-8'))
  const appVersion = appJson.expo.version
  const versionSuffix = appVersion.replace(/\./g, '_')

  const universalApkRename = async () => {
    const apkPath = apkFile[0]
    const newApkName = `heartio_andriod_${versionSuffix}.apk`
    const newApkPath = path.join(outDir, newApkName)
    await fs.rename(apkPath, newApkPath)
    console.log(`Renamed APK to: ${newApkName}`)
  }
  const multiPlatformApkRename = async () => {
    // arm64-v8a
    // armeabi-v7a
    const v8aApkPath = path.join(outDir, 'app-arm64-v8a-release.apk')
    const v7aApkPath = path.join(outDir, 'app-armeabi-v7a-release.apk')
    if (!fs.existsSync(v8aApkPath) || !fs.existsSync(v7aApkPath)) {
      throw new Error(
        `Multi-platform APKs not found in ${outDir}. Please ensure the build was successful.`,
      )
    }
    // remove v7aApkPath
    await fs.remove(v7aApkPath)
    // rename v8aApkPath
    const newApkName = `heartio_andriod_${versionSuffix}.apk`
    const newApkPath = path.join(outDir, newApkName)
    await fs.rename(v8aApkPath, newApkPath)
  }
  multiPlatformApkRename()
}

run()

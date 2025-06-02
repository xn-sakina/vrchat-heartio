import 'zx/globals'

const run = async () => {
  const outDir = path.join(
    __dirname,
    '../android/app/build/outputs/apk/release',
  )

  const apkFile = await globby([path.join(outDir, '*.apk')], {
    absolute: true,
  })

  if (apkFile?.length !== 1) {
    throw new Error(
      `Expected exactly one APK file in ${outDir}, found: ${apkFile.length}`,
    )
  }

  const apkPath = apkFile[0]

  const appJsonPath = path.join(__dirname, '../app.json')
  const appJson = JSON.parse(await fs.readFile(appJsonPath, 'utf-8'))
  const appVersion = appJson.expo.version
  const versionSuffix = appVersion.replace(/\./g, '_')

  const newApkName = `heartio_andriod_${versionSuffix}.apk`
  const newApkPath = path.join(outDir, newApkName)
  await fs.rename(apkPath, newApkPath)
  console.log(`Renamed APK to: ${newApkName}`)
}

run()

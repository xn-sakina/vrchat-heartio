const { withAppBuildGradle } = require('@expo/config-plugins')
const fs = require('fs')
const path = require('path')

module.exports = function withAndroidSignature(config) {
  return withAppBuildGradle(config, (config) => {
    if (config.modResults.language === 'groovy') {
      config.modResults.contents = setAndroidSignature(
        config.modResults.contents,
      )
    } else {
      throw new Error()
    }
    return config
  })
}

function setAndroidSignature(appBuildGradle) {
  if (!fs.existsSync(path.resolve(__dirname, '../credentials.json'))) {
    return appBuildGradle
  }
  const info = JSON.parse(
    fs.readFileSync(path.resolve(__dirname, '../credentials.json'), {
      encoding: 'utf8',
    }),
  )

  let output = appBuildGradle.replace(
    /(signingConfigs\s*\{)/,
    `$1
        release {
            storeFile file(${JSON.stringify(
              path.resolve(
                __dirname,
                '../',
                info.android.keystore.keystorePath,
              ),
            )})
            storePassword ${JSON.stringify(
              info.android.keystore.keystorePassword,
            )}
            keyAlias ${JSON.stringify(info.android.keystore.keyAlias)}
            keyPassword ${JSON.stringify(info.android.keystore.keyPassword)}
        }`,
  )

  output = output.replace(
    /(release\s*\{)[^}]*?signingConfig\s+signingConfigs\.debug/s,
    `$1
            signingConfig signingConfigs.release
`,
  )

  return output
}

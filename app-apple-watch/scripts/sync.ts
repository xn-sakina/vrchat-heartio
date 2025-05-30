import 'zx/globals'

const codeDir = '/Users/sakina/Documents/code/xcode/heartio/heartio'

const run = async () => {
  const targetDir = path.join(__dirname, '../heartio')

  if (fs.existsSync(targetDir)) {
    // remove
    fs.removeSync(targetDir)
  }

  // copy
  fs.copySync(codeDir, targetDir)

  // remove .git
  const gitDir = path.join(targetDir, '.git')
  if (fs.existsSync(gitDir)) {
    fs.removeSync(gitDir)
  }

  const projConfigFile = path.join(
    targetDir,
    './heartio.xcodeproj/project.pbxproj',
  )
  const projConfig = fs.readFileSync(projConfigFile, 'utf-8')
  // remove DEVELOPMENT_TEAM
  const newProjConfig = projConfig.replace(
    /DEVELOPMENT_TEAM = [0-9A-Z]{10};/g,
    '',
  )
  fs.writeFileSync(projConfigFile, newProjConfig)
}

run()

import { createConsola } from 'consola'

export const createLogger = (tag: string) => {
  return createConsola({
    formatOptions: {
      date: true,
    },
    defaults: {
      tag,
    },
  })
}

import AsyncStorage from '@react-native-async-storage/async-storage'

export class Storage {
  static async saveData(data: Record<string, any>) {
    try {
      const json = JSON.stringify(data)
      await AsyncStorage.setItem('appData', json)
    } catch (error) {
      console.error('Error saving data to storage:', error)
    }
  }

  static async loadData(): Promise<Record<string, any> | null> {
    try {
      const json = await AsyncStorage.getItem('appData')
      return json ? JSON.parse(json) : null
    } catch (error) {
      console.error('Error loading data from storage:', error)
      return null
    }
  }
}

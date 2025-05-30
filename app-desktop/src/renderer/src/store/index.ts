import { create } from 'zustand'

interface IGlobalData {
  // listening
  isListening: boolean
  // current heart rate
  currentHeartRate: number
}

export const useGlobalData = create<IGlobalData>((set) => ({
  isListening: false,
  currentHeartRate: 0,
}))

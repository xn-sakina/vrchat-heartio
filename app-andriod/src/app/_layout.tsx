import 'react-native-reanimated'

import { StatusBar } from 'expo-status-bar'
import Home from './home'
import { StyleSheet, View } from 'react-native'
import { GestureHandlerRootView } from 'react-native-gesture-handler'
import { Toaster } from 'sonner-native'
import { KeyboardProvider } from 'react-native-keyboard-controller'

export default function RootLayout() {
  return (
    <GestureHandlerRootView>
      <KeyboardProvider>
        <View style={styles.container}>
          <Home />
          <Toaster />
        </View>
        <StatusBar style="auto" />
      </KeyboardProvider>
    </GestureHandlerRootView>
  )
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#fff',
  },
})

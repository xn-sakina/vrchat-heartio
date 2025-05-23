import Foundation
import SwiftUI
import HealthKit

// MARK: - HeartRateManager
class HeartRateManager: NSObject, ObservableObject, HKWorkoutSessionDelegate, HKLiveWorkoutBuilderDelegate {
    private var healthStore = HKHealthStore()
    private var session: HKWorkoutSession?
    private var builder: HKLiveWorkoutBuilder?
    private var timer: Timer?
    private var isStopping = false
    private var isStarting = false

    @Published var currentHeartRate: Double? = nil
    @Published var isAuthorized: Bool = false
    @Published var isMeasuring: Bool = false
    @Published var isLoading: Bool = false
    @Published var showAuthAlert: Bool = false
    @Published var alertMessage: String = ""

    var onHttpFail: (() -> Void)?
    private var httpFailed: Bool = false
    private var authFailed: Bool = false

    override init() {
        super.init()
    }

    /// HealthKit Authorization Request
    func requestAuthorization(completion: (() -> Void)? = nil) {
        guard HKHealthStore.isHealthDataAvailable() else {
            print("Health data not available on this device.")
            self.isAuthorized = false
            self.alertMessage = "Health data is not available on this device."
            self.showAuthAlert = true
            completion?()
            return
        }
        let heartRateType = HKQuantityType.quantityType(forIdentifier: .heartRate)!
        let workoutType = HKObjectType.workoutType()
        healthStore.requestAuthorization(
            toShare: [workoutType],
            read: [heartRateType]
        ) { [weak self] (success, error) in
            DispatchQueue.main.async {
                self?.isAuthorized = success
                if success {
                    self?.authFailed = false
                    completion?()
                } else {
                    if !(self?.authFailed ?? false) {
                        self?.authFailed = true
                        self?.alertMessage = "Authorization failed. Please check Health permissions."
                        self?.showAuthAlert = true
                    }
                    self?.stopWorkout()
                    completion?()
                }
            }
        }
    }

    /// Start workout session for real-time heart rate with background support.
    func startWorkout() {
        // Prevent duplicate starts
        guard !isMeasuring, !isStarting, !isStopping else { return }
        isStarting = true
        isLoading = true
        // Reset error states for new session
        httpFailed = false
        authFailed = false
        currentHeartRate = nil

        guard HKHealthStore.isHealthDataAvailable() else {
            alertMessage = "Health data not supported on this device."
            showAuthAlert = true
            isLoading = false
            isStarting = false
            return
        }
        let heartRateType = HKQuantityType.quantityType(forIdentifier: .heartRate)!
        let workoutType = HKObjectType.workoutType()
        healthStore.getRequestStatusForAuthorization(toShare: [workoutType], read: [heartRateType]) { [weak self] status, error in
            DispatchQueue.main.async {
                guard let self = self else { return }
                if status == .shouldRequest {
                    self.requestAuthorization {
                        self._startWorkoutSession()
                    }
                    return
                }
                if !self.isAuthorized {
                    if !self.authFailed {
                        self.authFailed = true
                        self.alertMessage = "Authorization failed. Please check Health permissions."
                        self.showAuthAlert = true
                    }
                    self.isLoading = false
                    self.isStarting = false
                    self.stopWorkout()
                    return
                }
                self._startWorkoutSession()
            }
        }
    }

    /// Internal: configures and starts Workout Session and LiveWorkoutBuilder
    private func _startWorkoutSession() {
        let configuration = HKWorkoutConfiguration()
        configuration.activityType = .other
        configuration.locationType = .indoor

        do {
            session = try HKWorkoutSession(healthStore: healthStore, configuration: configuration)
            builder = session?.associatedWorkoutBuilder()
        } catch {
            print("Error starting workout: \(error.localizedDescription)")
            alertMessage = "Failed to start workout: \(error.localizedDescription)"
            showAuthAlert = true
            isLoading = false
            isStarting = false
            stopWorkout()
            return
        }

        session?.delegate = self
        builder?.delegate = self
        builder?.dataSource = HKLiveWorkoutDataSource(healthStore: healthStore, workoutConfiguration: configuration)

        session?.startActivity(with: Date())
        builder?.beginCollection(withStart: Date(), completion: { [weak self] (success, error) in
            DispatchQueue.main.async {
                guard let self = self else { return }
                self.isLoading = false
                self.isStarting = false
                if !success {
                    let errStr = error?.localizedDescription ?? "ERROR"
                    print("Failed to begin collection: \(errStr)")
                    self.alertMessage = "Cannot collect heart rate, check permissions."
                    self.showAuthAlert = true
                    self.isMeasuring = false
                    self.stopWorkout()
                    return
                }
                self.isMeasuring = true
                self.isStopping = false

                self.timer?.invalidate()
                self.timer = Timer.scheduledTimer(withTimeInterval: 1, repeats: true) { [weak self] _ in
                    self?.reportHeartRate()
                }
                // Enable timer in all run loop modes (background safe)
                RunLoop.current.add(self.timer!, forMode: .common)
            }
        })
    }

    /// Stop workout session and invalidate timer
    func stopWorkout() {
        // Prevent duplicate stops or racing with start
        if isStopping || isStarting { return }
        isStopping = true
        isLoading = false
        timer?.invalidate()
        timer = nil

        if let session = session, let builder = builder {
            if session.state == .running {
                builder.endCollection(withEnd: Date()) { [weak self] (_, error) in
                    self?.endSessionIfNeeded()
                }
            } else {
                endSessionIfNeeded()
            }
        } else {
            isMeasuring = false
            isStopping = false
        }
    }

    private func endSessionIfNeeded() {
        if let session = session, session.state != .ended {
            session.end()
        }
        builder = nil
        session = nil
        DispatchQueue.main.async {
            self.isMeasuring = false
            self.isStopping = false
            self.isLoading = false
        }
    }

    // MARK: - Real-time heart rate stream, only pick the latest value (NOT average)
    func workoutBuilder(_ workoutBuilder: HKLiveWorkoutBuilder, didCollectDataOf collectedTypes: Set<HKSampleType>) {
        if collectedTypes.contains(HKObjectType.quantityType(forIdentifier: .heartRate)!) {
            if let stats = workoutBuilder.statistics(for: HKQuantityType.quantityType(forIdentifier: .heartRate)!),
               let value = stats.mostRecentQuantity()?.doubleValue(for: HKUnit(from: "count/min")), value > 0 {
                DispatchQueue.main.async {
                    self.currentHeartRate = value
                }
            }
        }
    }

    /// HTTP reporting every second, always report the latest heart rate (not average)
    private func reportHeartRate() {
        guard isMeasuring else { return }
        guard let heartRate = currentHeartRate, heartRate > 0 else { return }
        let bpm = Int(heartRate)
        guard let url = URL(string: "http://mio.mac.internal:2333/heart?bpm=\(bpm)") else { return }
        var req = URLRequest(url: url)
        req.httpMethod = "GET"
        let task = URLSession.shared.dataTask(with: req) { [weak self] _, response, error in
            guard let self = self else { return }
            if let httpResponse = response as? HTTPURLResponse, httpResponse.statusCode != 200 {
                print("HTTP status code: \(httpResponse.statusCode)")
                DispatchQueue.main.async { self.handleHttpFail() }
            }
            if let error = error {
                print("HTTP error: \(error.localizedDescription)")
                DispatchQueue.main.async { self.handleHttpFail() }
            }
        }
        task.resume()
    }

    /// Handle HTTP failures, only prompt once and stop reporting
    private func handleHttpFail() {
        if !httpFailed {
            httpFailed = true
            stopWorkout()
            onHttpFail?()
        }
    }

    // MARK: - HKWorkoutSessionDelegate
    func workoutSession(_ workoutSession: HKWorkoutSession, didChangeTo toState: HKWorkoutSessionState, from fromState: HKWorkoutSessionState, date: Date) {
        print("Workout session changed from \(fromState.rawValue) to \(toState.rawValue)")
        if toState == .ended {
            builder?.finishWorkout(completion: { [weak self] (_, error) in
                DispatchQueue.main.async {
                    self?.isMeasuring = false
                    self?.isStopping = false
                    self?.isLoading = false
                }
            })
        }
    }
    func workoutSession(_ workoutSession: HKWorkoutSession, didFailWithError error: Error) {
        print("Workout session failed: \(error.localizedDescription)")
        DispatchQueue.main.async {
            self.isMeasuring = false
            self.isStopping = false
            self.isLoading = false
            self.alertMessage = "Workout session failed: \(error.localizedDescription)"
            self.showAuthAlert = true
            self.stopWorkout()
        }
    }
    func workoutSession(_ workoutSession: HKWorkoutSession, didGenerate event: HKWorkoutEvent) {}
    func workoutBuilderDidCollectEvent(_ workoutBuilder: HKLiveWorkoutBuilder) {}

    /// Only when not measuring, fetch the latest heart rate sample for UI init
    func fetchLatestHeartRate() {
        guard !isMeasuring else { return }
        let heartRateType = HKQuantityType.quantityType(forIdentifier: .heartRate)!
        let sortDescriptor = NSSortDescriptor(key: HKSampleSortIdentifierStartDate, ascending: false)
        let startDate = Calendar.current.date(byAdding: .hour, value: -1, to: Date())
        let predicate = HKQuery.predicateForSamples(withStart: startDate, end: Date(), options: [])

        let query = HKSampleQuery(sampleType: heartRateType, predicate: predicate, limit: 1, sortDescriptors: [sortDescriptor]) { [weak self] (_, samples, _) in
            guard let sample = samples?.first as? HKQuantitySample else { return }
            let hr = sample.quantity.doubleValue(for: HKUnit(from: "count/min"))
            DispatchQueue.main.async {
                self?.currentHeartRate = hr > 0 ? hr : nil
            }
        }
        healthStore.execute(query)
    }
}

// MARK: - ContentView
struct ContentView: View {
    @StateObject private var heartRateManager = HeartRateManager()
    @State private var showHttpFailAlert = false

    var body: some View {
        VStack(spacing: 10) {
            Text("Heart Rate")
                .font(.system(size: 20, weight: .semibold, design: .rounded))
                .foregroundColor(.white)
                .padding(.top, 2)

            ZStack {
                RoundedRectangle(cornerRadius: 22, style: .continuous)
                    .fill(Color(UIColor.black))
                    .frame(height: 60)
                    .shadow(radius: 2)

                if heartRateManager.isLoading {
                    ProgressView()
                        .progressViewStyle(CircularProgressViewStyle(tint: .gray))
                        .padding(.leading, -100)
                    Text("Loading…")
                        .font(.system(size: 18, weight: .medium))
                        .foregroundColor(.white)
                        .padding(.leading, 30)
                } else if let bpm = heartRateManager.currentHeartRate, bpm > 0 {
                    HStack(spacing: 6) {
                        Image(systemName: "heart.fill")
                            .foregroundColor(.red)
                        Text("\(Int(bpm))")
                            .font(.system(size: 45, weight: .heavy, design: .rounded))
                            .foregroundColor(.red)
                        Text("BPM")
                            .font(.system(size: 18, weight: .bold))
                            .foregroundColor(.green)
                            .padding(.leading, 3)
                    }
                } else {
                    Text("No Data")
                        .font(.system(size: 18, weight: .medium))
                        .foregroundColor(.white)
                }
            }
            .padding(.vertical, 2)

            Button(action: {
                // Prevent button spam
                if heartRateManager.isLoading { return }
                if heartRateManager.isMeasuring {
                    heartRateManager.stopWorkout()
                } else {
                    heartRateManager.startWorkout()
                }
            }) {
                if heartRateManager.isLoading {
                    HStack {
                        ProgressView()
                            .progressViewStyle(CircularProgressViewStyle(tint: .white))
                        Text(heartRateManager.isMeasuring ? "Stopping..." : "Starting...")
                            .font(.system(size: 20, weight: .semibold))
                    }
                    .frame(maxWidth: .infinity)
                    .padding(.vertical, 8)
                } else {
                    Label(
                        heartRateManager.isMeasuring ? "Stop" : "Start",
                        systemImage: heartRateManager.isMeasuring ? "stop.circle.fill" : "play.circle.fill"
                    )
                    .font(.system(size: 20, weight: .semibold))
                    .frame(maxWidth: .infinity)
                    .padding(.vertical, 8)
                }
            }
            // Disable when loading, preventing multiple tap
            .disabled(heartRateManager.isLoading || (!heartRateManager.isAuthorized && !heartRateManager.isMeasuring))
            .background(heartRateManager.isMeasuring ? Color.blue : Color.green)
            .foregroundColor(.white)
            .cornerRadius(16)
            .shadow(radius: heartRateManager.isMeasuring ? 1 : 3)
            .padding(.horizontal, 0)
            .padding(.top, 2)

            Spacer()
        }
        .padding([.horizontal, .bottom], 12)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(Color(UIColor.black))
        .onAppear {
            heartRateManager.isLoading = true
            heartRateManager.requestAuthorization {
                heartRateManager.fetchLatestHeartRate()
                heartRateManager.isLoading = false
            }
            heartRateManager.onHttpFail = {
                showHttpFailAlert = true
            }
        }
        .alert(isPresented: $heartRateManager.showAuthAlert) {
            Alert(
                title: Text("Message"),
                message: Text(heartRateManager.alertMessage),
                dismissButton: .default(Text("OK")) {
                    heartRateManager.showAuthAlert = false
                }
            )
        }
        .alert(isPresented: $showHttpFailAlert) {
            Alert(
                title: Text("Network Error"),
                message: Text("Failed to report heart rate. Reporting has been stopped."),
                dismissButton: .default(Text("OK")) {
                    showHttpFailAlert = false
                }
            )
        }
    }
}

#Preview {
    ContentView()
}

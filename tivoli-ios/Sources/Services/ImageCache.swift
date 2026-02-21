import UIKit

actor ImageCache {
    static let shared = ImageCache()

    private let cache = NSCache<NSString, UIImage>()
    private var inFlight: [String: Task<UIImage, Error>] = [:]
    private var prefetchInFlight: [String: Task<UIImage, Error>] = [:]

    private let mainSession: URLSession
    private let prefetchSession: URLSession

    private init() {
        cache.countLimit = 200
        cache.totalCostLimit = 100 * 1024 * 1024

        let mainConfig = URLSessionConfiguration.default
        mainConfig.httpMaximumConnectionsPerHost = 8
        mainSession = URLSession(configuration: mainConfig)

        let prefetchConfig = URLSessionConfiguration.default
        prefetchConfig.httpMaximumConnectionsPerHost = 3
        prefetchConfig.networkServiceType = .background
        prefetchSession = URLSession(configuration: prefetchConfig)
    }

    func prefetch(urls: [URL]) {
        for url in urls {
            let key = url.absoluteString as NSString
            guard cache.object(forKey: key) == nil,
                  inFlight[url.absoluteString] == nil,
                  prefetchInFlight[url.absoluteString] == nil else { continue }

            let session = prefetchSession
            let task = Task<UIImage, Error>(priority: .low) {
                let (data, _) = try await session.data(from: url)
                guard let image = UIImage(data: data) else {
                    throw URLError(.cannotDecodeContentData)
                }
                self.cache.setObject(image, forKey: key, cost: data.count)
                return image
            }
            prefetchInFlight[url.absoluteString] = task
            Task {
                _ = try? await task.value
                prefetchInFlight.removeValue(forKey: url.absoluteString)
            }
        }
    }

    func image(for url: URL) async throws -> UIImage {
        let key = url.absoluteString as NSString

        if let cached = cache.object(forKey: key) {
            return cached
        }

        // If already loading at high priority, await it
        if let existing = inFlight[url.absoluteString] {
            return try await existing.value
        }

        // Cancel any low-priority prefetch for this URL — we'll load it now
        if let prefetchTask = prefetchInFlight[url.absoluteString] {
            prefetchTask.cancel()
            prefetchInFlight.removeValue(forKey: url.absoluteString)
        }

        let session = mainSession
        let task = Task<UIImage, Error>(priority: .high) {
            let (data, _) = try await session.data(from: url)
            guard let image = UIImage(data: data) else {
                throw URLError(.cannotDecodeContentData)
            }
            self.cache.setObject(image, forKey: key, cost: data.count)
            return image
        }

        inFlight[url.absoluteString] = task
        let image = try await task.value
        inFlight.removeValue(forKey: url.absoluteString)
        return image
    }
}

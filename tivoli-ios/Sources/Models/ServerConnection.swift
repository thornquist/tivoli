import Foundation

struct ServerConnection: Codable, Identifiable, Hashable, Sendable {
    var id: String { url }
    let url: String
    let label: String
    let lastConnected: Date
    var useThumbnails: Bool
    var prefetchCount: Int

    init(url: String, label: String, lastConnected: Date, useThumbnails: Bool = true, prefetchCount: Int = 50) {
        self.url = url
        self.label = label
        self.lastConnected = lastConnected
        self.useThumbnails = useThumbnails
        self.prefetchCount = prefetchCount
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        url = try container.decode(String.self, forKey: .url)
        label = try container.decode(String.self, forKey: .label)
        lastConnected = try container.decode(Date.self, forKey: .lastConnected)
        useThumbnails = try container.decodeIfPresent(Bool.self, forKey: .useThumbnails) ?? true
        prefetchCount = try container.decodeIfPresent(Int.self, forKey: .prefetchCount) ?? 50
    }
}

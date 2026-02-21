import Foundation

struct ServerConnection: Codable, Identifiable, Hashable, Sendable {
    var id: String { url }
    let url: String
    let label: String
    let lastConnected: Date
}

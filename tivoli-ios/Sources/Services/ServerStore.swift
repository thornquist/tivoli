import Foundation

@Observable
@MainActor
final class ServerStore {
    private static let storageKey = "savedServers"

    var servers: [ServerConnection] {
        didSet { save() }
    }

    init() {
        if let data = UserDefaults.standard.data(forKey: Self.storageKey),
           let decoded = try? JSONDecoder().decode([ServerConnection].self, from: data)
        {
            self.servers = decoded.sorted { $0.lastConnected > $1.lastConnected }
        } else {
            self.servers = []
        }
    }

    func addOrUpdate(url: String) {
        let cleaned = url
            .trimmingCharacters(in: .whitespacesAndNewlines)
            .trimmingCharacters(in: CharacterSet(charactersIn: "/"))
        let label = URL(string: cleaned)?.host ?? cleaned
        let connection = ServerConnection(
            url: cleaned, label: label, lastConnected: Date()
        )

        servers.removeAll { $0.url == cleaned }
        servers.insert(connection, at: 0)

        if servers.count > 10 {
            servers = Array(servers.prefix(10))
        }
    }

    func remove(_ connection: ServerConnection) {
        servers.removeAll { $0.id == connection.id }
    }

    private func save() {
        if let data = try? JSONEncoder().encode(servers) {
            UserDefaults.standard.set(data, forKey: Self.storageKey)
        }
    }
}

import Foundation

struct ModelRef: Codable, Identifiable, Hashable, Sendable {
    var id: String { uuid }
    let uuid: String
    let name: String
}

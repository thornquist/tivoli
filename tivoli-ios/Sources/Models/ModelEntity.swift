import Foundation

struct ModelEntity: Codable, Identifiable, Hashable, Sendable {
    var id: String { uuid }
    let uuid: String
    let name: String
    let collection: String
}

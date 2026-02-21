import Foundation

struct TagRef: Codable, Identifiable, Hashable, Sendable {
    var id: String { uuid }
    let uuid: String
    let name: String
    let group: String
}

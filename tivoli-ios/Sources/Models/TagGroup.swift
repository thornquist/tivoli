import Foundation

struct TagGroup: Codable, Identifiable, Hashable, Sendable {
    var id: String { uuid }
    let uuid: String
    let name: String
    let tags: [Tag]
}

struct Tag: Codable, Identifiable, Hashable, Sendable {
    var id: String { uuid }
    let uuid: String
    let name: String
}

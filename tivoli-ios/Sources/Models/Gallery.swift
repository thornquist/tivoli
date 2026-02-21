import Foundation

struct Gallery: Codable, Identifiable, Hashable, Sendable {
    var id: String { "\(collection)/\(name)" }
    let name: String
    let collection: String
    let imageCount: Int

    enum CodingKeys: String, CodingKey {
        case name, collection
        case imageCount = "image_count"
    }
}

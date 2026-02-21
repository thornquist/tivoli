import Foundation

struct Collection: Codable, Identifiable, Hashable, Sendable {
    var id: String { name }
    let name: String
    let imageCount: Int
    let galleryCount: Int

    enum CodingKeys: String, CodingKey {
        case name
        case imageCount = "image_count"
        case galleryCount = "gallery_count"
    }
}

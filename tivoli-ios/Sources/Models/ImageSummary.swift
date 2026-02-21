import Foundation

struct ImageSummary: Codable, Identifiable, Hashable, Sendable {
    var id: String { uuid }
    let uuid: String
    let path: String
    let collection: String
    let gallery: String
    let width: Int
    let height: Int

    var aspectRatio: CGFloat {
        guard height > 0 else { return 1 }
        return CGFloat(width) / CGFloat(height)
    }
}

struct FilterOptions: Codable, Sendable {
    let imageCount: Int
    let collections: [String]
    let galleries: [GallerySummary]
    let models: [ModelEntity]
    let tags: [TagRef]

    enum CodingKeys: String, CodingKey {
        case imageCount = "image_count"
        case collections, galleries, models, tags
    }
}

struct ImageDetail: Codable, Sendable {
    let uuid: String
    let path: String
    let collection: String
    let gallery: String
    let width: Int
    let height: Int
    let models: [ModelEntity]
    let tags: [TagRef]
}

struct GallerySummary: Codable, Sendable {
    let name: String
    let collection: String
    let imageCount: Int

    enum CodingKeys: String, CodingKey {
        case name, collection
        case imageCount = "image_count"
    }
}

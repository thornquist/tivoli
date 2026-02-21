import Foundation

struct ImageSummary: Codable, Identifiable, Hashable, Sendable {
    var id: String { uuid }
    let uuid: String
    let path: String
    let collection: String
    let gallery: String
    let width: Int
    let height: Int
    let fileSize: Int

    var aspectRatio: CGFloat {
        guard height > 0 else { return 1 }
        return CGFloat(width) / CGFloat(height)
    }

    enum CodingKeys: String, CodingKey {
        case uuid, path, collection, gallery, width, height
        case fileSize = "file_size"
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
    let fileSize: Int
    let models: [ModelEntity]
    let tags: [TagRef]

    enum CodingKeys: String, CodingKey {
        case uuid, path, collection, gallery, width, height, models, tags
        case fileSize = "file_size"
    }
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

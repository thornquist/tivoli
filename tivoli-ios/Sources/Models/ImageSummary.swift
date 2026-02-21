import Foundation

struct ImageSummary: Codable, Identifiable, Hashable, Sendable {
    var id: String { uuid }
    let uuid: String
    let path: String
    let collection: String
    let gallery: String
    let width: Int
    let height: Int
    var models: [ModelRef]
    var tags: [TagRef]

    var aspectRatio: CGFloat {
        guard height > 0 else { return 1 }
        return CGFloat(width) / CGFloat(height)
    }
}

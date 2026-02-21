import Foundation

struct SearchRequest: Codable, Sendable {
    let filters: [FilterClause]
}

struct FilterClause: Codable, Hashable, Identifiable, Sendable {
    let id: UUID
    var field: FilterField
    var op: FilterOp
    var value: FilterValue

    enum CodingKeys: String, CodingKey {
        case field, op, value
    }

    init(field: FilterField, op: FilterOp, value: FilterValue) {
        self.id = UUID()
        self.field = field
        self.op = op
        self.value = value
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        self.id = UUID()
        self.field = try container.decode(FilterField.self, forKey: .field)
        self.op = try container.decode(FilterOp.self, forKey: .op)
        self.value = try container.decode(FilterValue.self, forKey: .value)
    }
}

enum FilterField: String, Codable, CaseIterable, Sendable {
    case collection
    case gallery
    case models
    case tags
}

enum FilterOp: String, Codable, CaseIterable, Sendable {
    case eq
    case anyOf = "any_of"
    case allOf = "all_of"
    case exact
    case noneOf = "none_of"
}

enum FilterValue: Codable, Hashable, Sendable {
    case single(String)
    case multiple([String])

    func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()
        switch self {
        case .single(let s): try container.encode(s)
        case .multiple(let arr): try container.encode(arr)
        }
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        if let s = try? container.decode(String.self) {
            self = .single(s)
        } else {
            self = .multiple(try container.decode([String].self))
        }
    }
}

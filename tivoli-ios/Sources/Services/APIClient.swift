import Foundation
import SwiftUI

enum APIError: LocalizedError, Sendable {
    case invalidURL
    case networkError(String)
    case decodingError(String)
    case serverError(statusCode: Int, message: String)

    var errorDescription: String? {
        switch self {
        case .invalidURL: "Invalid server URL"
        case .networkError(let msg): "Network error: \(msg)"
        case .decodingError(let msg): "Data error: \(msg)"
        case .serverError(let code, let msg): "Server error (\(code)): \(msg)"
        }
    }
}

@Observable
@MainActor
final class APIClient {
    let baseURL: URL
    private let session: URLSession
    private let decoder: JSONDecoder

    init(baseURL: URL) {
        self.baseURL = baseURL
        let config = URLSessionConfiguration.default
        config.timeoutIntervalForRequest = 15
        self.session = URLSession(configuration: config)
        self.decoder = JSONDecoder()
    }

    // MARK: - Endpoints

    func fetchCollections() async throws -> [Collection] {
        try await get("/collections")
    }

    func fetchGalleries(collection: String? = nil) async throws -> [Gallery] {
        var path = "/galleries"
        if let collection {
            path += "?collection=\(collection)"
        }
        return try await get(path)
    }

    func fetchModels(collection: String? = nil) async throws -> [ModelEntity] {
        var path = "/models"
        if let collection {
            path += "?collection=\(collection)"
        }
        return try await get(path)
    }

    func fetchTags() async throws -> [TagGroup] {
        try await get("/tags")
    }

    func searchImages(filters: [FilterClause]) async throws -> [ImageSummary] {
        let body = SearchRequest(filters: filters)
        return try await post("/images/search", body: body)
    }

    func updateImageTags(imageUUID: String, tagUUIDs: [String]) async throws {
        let body = ["tag_uuids": tagUUIDs]
        try await putNoContent("/images/\(imageUUID)/tags", body: body)
    }

    func imageURL(uuid: String) -> URL {
        baseURL.appendingPathComponent("images/\(uuid)/file")
    }

    func testConnection() async throws {
        let _: [Collection] = try await get("/collections")
    }

    // MARK: - Private

    private func get<T: Decodable & Sendable>(_ path: String) async throws(APIError) -> T {
        guard let url = URL(string: path, relativeTo: baseURL) else {
            throw .invalidURL
        }
        do {
            let (data, response) = try await session.data(from: url)
            try validateResponse(response)
            return try decoder.decode(T.self, from: data)
        } catch let error as APIError {
            throw error
        } catch let error as DecodingError {
            throw .decodingError(error.localizedDescription)
        } catch {
            throw .networkError(error.localizedDescription)
        }
    }

    private func post<T: Decodable & Sendable, B: Encodable & Sendable>(
        _ path: String, body: B
    ) async throws(APIError) -> T {
        guard let url = URL(string: path, relativeTo: baseURL) else {
            throw .invalidURL
        }
        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        do {
            request.httpBody = try JSONEncoder().encode(body)
            let (data, response) = try await session.data(for: request)
            try validateResponse(response)
            return try decoder.decode(T.self, from: data)
        } catch let error as APIError {
            throw error
        } catch let error as DecodingError {
            throw .decodingError(error.localizedDescription)
        } catch {
            throw .networkError(error.localizedDescription)
        }
    }

    private func putNoContent<B: Encodable & Sendable>(
        _ path: String, body: B
    ) async throws(APIError) {
        guard let url = URL(string: path, relativeTo: baseURL) else {
            throw .invalidURL
        }
        var request = URLRequest(url: url)
        request.httpMethod = "PUT"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        do {
            request.httpBody = try JSONEncoder().encode(body)
            let (_, response) = try await session.data(for: request)
            try validateResponse(response)
        } catch let error as APIError {
            throw error
        } catch {
            throw .networkError(error.localizedDescription)
        }
    }

    private func validateResponse(_ response: URLResponse) throws(APIError) {
        guard let http = response as? HTTPURLResponse else { return }
        guard (200...299).contains(http.statusCode) else {
            throw .serverError(statusCode: http.statusCode, message: "Request failed")
        }
    }
}

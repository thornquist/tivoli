import SwiftUI

struct CachedAsyncImage: View {
    let url: URL
    @State private var image: UIImage?
    @State private var isLoading = true

    var body: some View {
        GeometryReader { geo in
            Group {
                if let image {
                    Image(uiImage: image)
                        .resizable()
                        .aspectRatio(contentMode: .fill)
                        .frame(width: geo.size.width, height: geo.size.height)
                        .clipped()
                } else if isLoading {
                    Color(.systemGray6)
                        .overlay { ProgressView().scaleEffect(0.7) }
                } else {
                    Color(.systemGray6)
                        .overlay {
                            Image(systemName: "photo")
                                .foregroundStyle(.quaternary)
                        }
                }
            }
        }
        .task(id: url) {
            isLoading = true
            do {
                image = try await ImageCache.shared.image(for: url)
            } catch {
                image = nil
            }
            isLoading = false
        }
    }
}

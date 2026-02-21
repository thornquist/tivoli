import SwiftUI

struct WaterfallGrid<Content: View>: View {
    let images: [ImageSummary]
    let columnCount: Int
    let spacing: CGFloat
    let prefetchCount: Int
    let imageURL: (ImageSummary) -> URL
    @ViewBuilder let content: (Int, ImageSummary) -> Content

    var body: some View {
        GeometryReader { geo in
            let containerWidth = geo.size.width
            let columnWidth = (containerWidth - spacing * CGFloat(columnCount - 1)) / CGFloat(columnCount)
            let layout = computeLayout(columnWidth: columnWidth)

            ScrollView {
                HStack(alignment: .top, spacing: spacing) {
                    ForEach(0..<columnCount, id: \.self) { col in
                        LazyVStack(spacing: spacing) {
                            ForEach(layout.columns[col]) { item in
                                content(item.globalIndex, images[item.globalIndex])
                                    .frame(width: columnWidth, height: item.height)
                                    .clipped()
                                    .onAppear { prefetch(from: item.globalIndex) }
                            }
                        }
                    }
                }
            }
        }
    }

    private func prefetch(from index: Int) {
        let start = index + 1
        let end = min(start + prefetchCount, images.count)
        guard start < end else { return }
        let urls = (start..<end).map { imageURL(images[$0]) }
        Task { await ImageCache.shared.prefetch(urls: urls) }
    }

    private struct Layout {
        let columns: [[ColumnItem]]
        let totalHeight: CGFloat
    }

    private func computeLayout(columnWidth: CGFloat) -> Layout {
        guard columnWidth > 0 else {
            return Layout(
                columns: Array(repeating: [], count: columnCount),
                totalHeight: 0
            )
        }
        var columns: [[ColumnItem]] = Array(repeating: [], count: columnCount)
        var columnHeights = Array(repeating: CGFloat.zero, count: columnCount)

        for (index, image) in images.enumerated() {
            let shortestColumn = columnHeights.enumerated()
                .min(by: { $0.element < $1.element })!.offset
            let itemHeight = columnWidth / image.aspectRatio

            columns[shortestColumn].append(
                ColumnItem(globalIndex: index, height: itemHeight)
            )
            columnHeights[shortestColumn] += itemHeight + spacing
        }

        let maxHeight = columnHeights.max() ?? 0
        return Layout(columns: columns, totalHeight: maxHeight)
    }
}

private struct ColumnItem: Identifiable {
    let globalIndex: Int
    let height: CGFloat
    var id: Int { globalIndex }
}

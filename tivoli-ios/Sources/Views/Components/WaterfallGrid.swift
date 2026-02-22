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
                                    .onAppear { prefetch(from: item.globalIndex, layout: layout) }
                            }
                        }
                    }
                }
            }
        }
    }

    private func prefetch(from index: Int, layout: Layout) {
        guard let pos = layout.visualPosition[index] else { return }
        let start = pos + 1
        let end = min(start + prefetchCount, layout.visualOrder.count)
        guard start < end else { return }
        let urls = (start..<end).map { imageURL(images[layout.visualOrder[$0]]) }
        Task { await ImageCache.shared.prefetch(urls: urls) }
    }

    private struct Layout {
        let columns: [[ColumnItem]]
        let totalHeight: CGFloat
        let visualOrder: [Int]          // globalIndex values sorted by y-position
        let visualPosition: [Int: Int]  // globalIndex → position in visualOrder
    }

    private func computeLayout(columnWidth: CGFloat) -> Layout {
        guard columnWidth > 0 else {
            return Layout(
                columns: Array(repeating: [], count: columnCount),
                totalHeight: 0,
                visualOrder: [],
                visualPosition: [:]
            )
        }
        var columns: [[ColumnItem]] = Array(repeating: [], count: columnCount)
        var columnHeights = Array(repeating: CGFloat.zero, count: columnCount)
        var items: [(globalIndex: Int, yPosition: CGFloat)] = []

        for (index, image) in images.enumerated() {
            let shortestColumn = columnHeights.enumerated()
                .min(by: { $0.element < $1.element })!.offset
            let itemHeight = columnWidth / image.aspectRatio
            let yPos = columnHeights[shortestColumn]

            columns[shortestColumn].append(
                ColumnItem(globalIndex: index, height: itemHeight)
            )
            items.append((globalIndex: index, yPosition: yPos))
            columnHeights[shortestColumn] += itemHeight + spacing
        }

        items.sort { $0.yPosition < $1.yPosition }
        let visualOrder = items.map(\.globalIndex)
        var visualPosition: [Int: Int] = [:]
        for (pos, globalIndex) in visualOrder.enumerated() {
            visualPosition[globalIndex] = pos
        }

        let maxHeight = columnHeights.max() ?? 0
        return Layout(
            columns: columns,
            totalHeight: maxHeight,
            visualOrder: visualOrder,
            visualPosition: visualPosition
        )
    }
}

private struct ColumnItem: Identifiable {
    let globalIndex: Int
    let height: CGFloat
    var id: Int { globalIndex }
}

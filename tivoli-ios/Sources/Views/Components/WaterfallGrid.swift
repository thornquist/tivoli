import SwiftUI

struct WaterfallGrid<Content: View>: View {
    let images: [ImageSummary]
    let columnCount: Int
    let spacing: CGFloat
    @ViewBuilder let content: (Int, ImageSummary) -> Content

    @State private var viewWidth: CGFloat = 0

    var body: some View {
        GeometryReader { geo in
            let columnWidth = (geo.size.width - spacing * CGFloat(columnCount - 1)) / CGFloat(columnCount)
            let layout = computeLayout(columnWidth: columnWidth)

            ZStack(alignment: .topLeading) {
                ForEach(layout.items) { item in
                    content(item.index, images[item.index])
                        .frame(width: columnWidth, height: item.height)
                        .clipped()
                        .offset(x: item.x, y: item.y)
                }
            }
            .frame(width: geo.size.width, height: layout.totalHeight, alignment: .topLeading)
            .onAppear { viewWidth = geo.size.width }
            .onChange(of: geo.size.width) { _, w in viewWidth = w }
        }
        .frame(height: estimateTotalHeight())
    }

    private func computeLayout(columnWidth: CGFloat) -> LayoutResult {
        var columnHeights = Array(repeating: CGFloat.zero, count: columnCount)
        var items: [LayoutItem] = []

        for (index, image) in images.enumerated() {
            let shortestColumn = columnHeights.enumerated().min(by: { $0.element < $1.element })!.offset
            let itemHeight = columnWidth / image.aspectRatio
            let x = CGFloat(shortestColumn) * (columnWidth + spacing)
            let y = columnHeights[shortestColumn]

            items.append(LayoutItem(index: index, x: x, y: y, height: itemHeight))
            columnHeights[shortestColumn] += itemHeight + spacing
        }

        let totalHeight = columnHeights.max() ?? 0
        return LayoutResult(items: items, totalHeight: totalHeight)
    }

    private func estimateTotalHeight() -> CGFloat {
        guard !images.isEmpty, viewWidth > 0 else { return 0 }
        let avgAspect = images.reduce(0.0) { $0 + $1.aspectRatio } / CGFloat(images.count)
        let estimatedRowHeight = viewWidth / CGFloat(columnCount) / avgAspect
        let rows = ceil(CGFloat(images.count) / CGFloat(columnCount))
        return rows * (estimatedRowHeight + spacing)
    }
}

private struct LayoutItem: Identifiable {
    let index: Int
    let x: CGFloat
    let y: CGFloat
    let height: CGFloat
    var id: Int { index }
}

private struct LayoutResult {
    let items: [LayoutItem]
    let totalHeight: CGFloat
}

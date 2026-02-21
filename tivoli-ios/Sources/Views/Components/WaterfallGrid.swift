import SwiftUI

struct WaterfallGrid<Content: View>: View {
    let images: [ImageSummary]
    let columnCount: Int
    let spacing: CGFloat
    @ViewBuilder let content: (Int, ImageSummary) -> Content

    @State private var containerWidth: CGFloat = 0

    var body: some View {
        let columnWidth = containerWidth > 0
            ? (containerWidth - spacing * CGFloat(columnCount - 1)) / CGFloat(columnCount)
            : 0
        let columns = assignColumns(columnWidth: columnWidth)

        HStack(alignment: .top, spacing: spacing) {
            ForEach(0..<columnCount, id: \.self) { col in
                LazyVStack(spacing: spacing) {
                    ForEach(columns[col]) { item in
                        content(item.globalIndex, images[item.globalIndex])
                            .frame(width: columnWidth, height: item.height)
                            .clipped()
                    }
                }
            }
        }
        .background(
            GeometryReader { geo in
                Color.clear.preference(key: ContainerWidthKey.self, value: geo.size.width)
            }
        )
        .onPreferenceChange(ContainerWidthKey.self) { containerWidth = $0 }
    }

    private func assignColumns(columnWidth: CGFloat) -> [[ColumnItem]] {
        guard columnWidth > 0 else {
            return Array(repeating: [], count: columnCount)
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

        return columns
    }
}

private struct ColumnItem: Identifiable {
    let globalIndex: Int
    let height: CGFloat
    var id: Int { globalIndex }
}

private struct ContainerWidthKey: PreferenceKey {
    nonisolated(unsafe) static var defaultValue: CGFloat = 0
    static func reduce(value: inout CGFloat, nextValue: () -> CGFloat) {
        value = nextValue()
    }
}

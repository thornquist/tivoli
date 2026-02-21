import SwiftUI

struct TagEditorView: View {
    @Binding var image: ImageSummary
    @Environment(APIClient.self) private var api
    @State private var tagGroups: [TagGroup] = []
    @State private var isLoading = true

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            // Models
            if !image.models.isEmpty {
                HStack {
                    Text("Models:")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                    ForEach(image.models) { model in
                        Text(model.name.capitalized)
                            .font(.caption.bold())
                    }
                }
            }

            if isLoading {
                ProgressView()
                    .frame(maxWidth: .infinity)
            } else {
                ScrollView {
                    VStack(alignment: .leading, spacing: 16) {
                        ForEach(tagGroups) { group in
                            VStack(alignment: .leading, spacing: 6) {
                                Text(group.name.capitalized)
                                    .font(.caption)
                                    .foregroundStyle(.secondary)

                                FlowLayout(spacing: 6) {
                                    ForEach(group.tags) { tag in
                                        let isSelected = image.tags.contains {
                                            $0.uuid == tag.uuid
                                        }
                                        Button {
                                            Task { await toggleTag(tag, group: group.name) }
                                        } label: {
                                            Text(tag.name.replacing("-", with: " "))
                                                .font(.caption)
                                                .padding(.horizontal, 10)
                                                .padding(.vertical, 6)
                                                .background(
                                                    isSelected ? Color.accentColor : Color.clear,
                                                    in: .capsule
                                                )
                                                .foregroundStyle(
                                                    isSelected ? .white : .primary
                                                )
                                                .overlay(
                                                    Capsule().stroke(
                                                        .secondary.opacity(0.3), lineWidth: 1
                                                    )
                                                )
                                        }
                                        .buttonStyle(.plain)
                                    }
                                }
                            }
                        }
                    }
                }
                .frame(maxHeight: 300)
            }
        }
        .padding()
        .background(.ultraThinMaterial, in: .rect(cornerRadius: 20))
        .padding()
        .task { await loadTags() }
    }

    private func loadTags() async {
        do {
            tagGroups = try await api.fetchTags()
        } catch {
            // error state
        }
        isLoading = false
    }

    private func toggleTag(_ tag: Tag, group: String) async {
        // Optimistic update
        if let idx = image.tags.firstIndex(where: { $0.uuid == tag.uuid }) {
            image.tags.remove(at: idx)
        } else {
            image.tags.append(TagRef(uuid: tag.uuid, name: tag.name, group: group))
        }

        let tagUUIDs = image.tags.map(\.uuid)
        do {
            try await api.updateImageTags(imageUUID: image.uuid, tagUUIDs: tagUUIDs)
        } catch {
            // Revert on failure — reload from server would be better
        }
    }
}

// MARK: - FlowLayout

struct FlowLayout: Layout {
    var spacing: CGFloat = 6

    func sizeThatFits(proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) -> CGSize {
        let rows = computeRows(proposal: proposal, subviews: subviews)
        var height: CGFloat = 0
        for (i, row) in rows.enumerated() {
            let rowHeight = row.map { $0.sizeThatFits(.unspecified).height }.max() ?? 0
            height += rowHeight
            if i > 0 { height += spacing }
        }
        return CGSize(width: proposal.width ?? 0, height: height)
    }

    func placeSubviews(
        in bounds: CGRect, proposal: ProposedViewSize, subviews: Subviews, cache: inout ()
    ) {
        let rows = computeRows(proposal: proposal, subviews: subviews)
        var y = bounds.minY
        for (i, row) in rows.enumerated() {
            if i > 0 { y += spacing }
            var x = bounds.minX
            let rowHeight = row.map { $0.sizeThatFits(.unspecified).height }.max() ?? 0
            for subview in row {
                let size = subview.sizeThatFits(.unspecified)
                subview.place(at: CGPoint(x: x, y: y), proposal: ProposedViewSize(size))
                x += size.width + spacing
            }
            y += rowHeight
        }
    }

    private func computeRows(
        proposal: ProposedViewSize, subviews: Subviews
    ) -> [[LayoutSubviews.Element]] {
        let maxWidth = proposal.width ?? .infinity
        var rows: [[LayoutSubviews.Element]] = [[]]
        var currentWidth: CGFloat = 0

        for subview in subviews {
            let size = subview.sizeThatFits(.unspecified)
            if currentWidth + size.width > maxWidth, !rows[rows.count - 1].isEmpty {
                rows.append([])
                currentWidth = 0
            }
            rows[rows.count - 1].append(subview)
            currentWidth += size.width + spacing
        }
        return rows
    }
}

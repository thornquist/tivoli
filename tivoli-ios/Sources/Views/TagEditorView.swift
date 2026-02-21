import SwiftUI

struct TagEditorView: View {
    let imageUUID: String
    @Environment(APIClient.self) private var api
    @State private var tagGroups: [TagGroup] = []
    @State private var imageDetail: ImageDetail?
    @State private var isLoading = true

    private var currentTagUUIDs: Set<String> {
        Set(imageDetail?.tags.map(\.uuid) ?? [])
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            // Models
            if let models = imageDetail?.models, !models.isEmpty {
                HStack {
                    Text("Models:")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                    ForEach(models) { model in
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
                                        let isSelected = currentTagUUIDs.contains(tag.uuid)
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
        .task { await loadData() }
    }

    private func loadData() async {
        async let detail = api.fetchImageDetail(uuid: imageUUID)
        async let groups = api.fetchTags()
        do {
            imageDetail = try await detail
            tagGroups = try await groups
        } catch {
            // error state
        }
        isLoading = false
    }

    private func toggleTag(_ tag: Tag, group: String) async {
        guard let detail = imageDetail else { return }

        // Optimistic update
        if let idx = detail.tags.firstIndex(where: { $0.uuid == tag.uuid }) {
            var tags = detail.tags
            tags.remove(at: idx)
            imageDetail = ImageDetail(
                uuid: detail.uuid, path: detail.path,
                collection: detail.collection, gallery: detail.gallery,
                width: detail.width, height: detail.height,
                models: detail.models, tags: tags
            )
        } else {
            var tags = detail.tags
            tags.append(TagRef(uuid: tag.uuid, name: tag.name, group: group))
            imageDetail = ImageDetail(
                uuid: detail.uuid, path: detail.path,
                collection: detail.collection, gallery: detail.gallery,
                width: detail.width, height: detail.height,
                models: detail.models, tags: tags
            )
        }

        let tagUUIDs = imageDetail?.tags.map(\.uuid) ?? []
        do {
            try await api.updateImageTags(imageUUID: imageUUID, tagUUIDs: tagUUIDs)
        } catch {
            // Revert on failure by reloading
            imageDetail = try? await api.fetchImageDetail(uuid: imageUUID)
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

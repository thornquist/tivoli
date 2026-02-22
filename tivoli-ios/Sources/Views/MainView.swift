import SwiftUI

struct MainView: View {
    let useThumbnails: Bool
    let prefetchCount: Int

    @Environment(APIClient.self) private var api

    // Filter data
    @State private var collections: [Collection] = []
    @State private var allModels: [ModelEntity] = []
    @State private var tagGroups: [TagGroup] = []

    // Filter selections
    @State private var selectedCollection: String?
    @State private var selectedModelUUIDs: Set<String> = []
    @State private var modelOp: FilterOp = .allOf
    @State private var selectedTagsByGroup: [String: Set<String>] = [:]
    @State private var tagOpByGroup: [String: FilterOp] = [:]

    // Filter options (from server)
    @State private var filterOptions: FilterOptions?

    // Results
    @State private var images: [ImageSummary] = []
    @State private var isLoadingData = true
    @State private var isSearching = false
    @State private var isLoadingImages = false
    @State private var selectedIndex: Int?
    @State private var searchTask: Task<Void, Never>?

    // Navigation
    @State private var currentPage = 0

    private var imageCount: Int { filterOptions?.imageCount ?? 0 }

    private var availableCollections: Set<String> {
        Set(filterOptions?.collections ?? [])
    }

    private var availableModelUUIDs: Set<String> {
        Set(filterOptions?.models.map(\.uuid) ?? [])
    }

    private var availableTagUUIDs: Set<String> {
        Set(filterOptions?.tags.map(\.uuid) ?? [])
    }

    private var hasActiveFilters: Bool {
        selectedCollection != nil
            || !selectedModelUUIDs.isEmpty
            || selectedTagsByGroup.values.contains(where: { !$0.isEmpty })
    }

    var body: some View {
        Group {
            if isLoadingData {
                ProgressView()
            } else {
                TabView(selection: $currentPage) {
                    filterPage.tag(0)
                    gridPage.tag(1)
                }
                .tabViewStyle(.page(indexDisplayMode: .never))
            }
        }
        .background(currentPage == 0 ? Color(.systemBackground) : .black)
        .task { await loadFilterData() }
        .onChange(of: selectedCollection) { _, _ in debouncedSearch() }
        .onChange(of: selectedModelUUIDs) { _, _ in debouncedSearch() }
        .onChange(of: modelOp) { _, _ in debouncedSearch() }
        .onChange(of: selectedTagsByGroup) { _, _ in debouncedSearch() }
        .onChange(of: tagOpByGroup) { _, _ in debouncedSearch() }
        .onChange(of: currentPage) { _, newPage in
            if newPage == 1 {
                Task { await fetchImages() }
            }
        }
        .fullScreenCover(item: $selectedIndex) { index in
            ImagePagerView(images: $images, initialIndex: index)
                .environment(api)
        }
    }

    // MARK: - Filter Page

    private var filterPage: some View {
        VStack(spacing: 0) {
            filterToolbar

            ScrollView {
                VStack(spacing: 0) {
                    filterPanel
                    Spacer(minLength: 32)
                    resultFooter
                    Spacer(minLength: 32)
                }
            }
        }
    }

    private var filterToolbar: some View {
        HStack {
            if hasActiveFilters {
                Button {
                    withAnimation(.snappy) {
                        selectedCollection = nil
                        selectedModelUUIDs = []
                        modelOp = .allOf
                        selectedTagsByGroup = [:]
                        tagOpByGroup = [:]
                    }
                } label: {
                    Text("Clear")
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                }
            }
            Spacer()
        }
        .padding(.horizontal, 20)
        .padding(.top, 8)
        .padding(.bottom, 4)
    }

    private var resultFooter: some View {
        VStack(spacing: 16) {
            if isSearching {
                ProgressView()
                Text("Searching...")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
            } else {
                Text("\(imageCount)")
                    .font(.system(size: 48, weight: .bold, design: .rounded))
                    .foregroundStyle(.primary)
                Text(imageCount == 1 ? "image" : "images")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .textCase(.uppercase)
                    .tracking(1)

                if imageCount > 0 {
                    Button {
                        withAnimation { currentPage = 1 }
                    } label: {
                        Text("View Results")
                            .font(.headline)
                            .frame(maxWidth: .infinity)
                            .frame(height: 50)
                            .background(Color.accentColor, in: .rect(cornerRadius: 12))
                            .foregroundStyle(.white)
                    }
                    .padding(.horizontal, 32)
                    .padding(.top, 8)
                }
            }
        }
        .frame(maxWidth: .infinity)
    }

    // MARK: - Grid Page

    private var gridPage: some View {
        ZStack {
            Color.black.ignoresSafeArea()

            if isLoadingImages {
                ProgressView()
                    .tint(.white)
            } else {
                WaterfallGrid(images: images, columnCount: 3, spacing: 2, prefetchCount: prefetchCount, imageURL: { gridImageURL($0.uuid) }) { index, image in
                    Button {
                        selectedIndex = index
                    } label: {
                        CachedAsyncImage(url: gridImageURL(image.uuid))
                            .aspectRatio(image.aspectRatio, contentMode: .fill)
                    }
                    .buttonStyle(.plain)
                }
                .padding(2)
            }
        }
    }

    // MARK: - Filter Panel

    private var filterPanel: some View {
        VStack(alignment: .leading, spacing: 20) {
            filterSection("Studio") {
                FlowLayout(spacing: 8) {
                    ForEach(collections) { col in
                        let isSelected = selectedCollection == col.name
                        FilterChip(
                            label: col.name.replacing("-", with: " ").capitalized,
                            isSelected: isSelected,
                            isDisabled: !isSelected && !availableCollections.contains(col.name)
                        ) {
                            withAnimation(.snappy) {
                                selectedCollection = selectedCollection == col.name ? nil : col.name
                            }
                        }
                    }
                }
            }

            if !allModels.isEmpty {
                filterSection("Models", op: $modelOp, ops: [.allOf, .exact]) {
                    FlowLayout(spacing: 6) {
                        ForEach(allModels) { model in
                            let isSelected = selectedModelUUIDs.contains(model.uuid)
                            FilterChip(
                                label: model.name.capitalized,
                                isSelected: isSelected,
                                isDisabled: !isSelected && !availableModelUUIDs.contains(model.uuid)
                            ) {
                                withAnimation(.snappy) {
                                    if selectedModelUUIDs.contains(model.uuid) {
                                        selectedModelUUIDs.remove(model.uuid)
                                    } else {
                                        selectedModelUUIDs.insert(model.uuid)
                                    }
                                }
                            }
                        }
                    }
                }
            }

            ForEach(tagGroups) { group in
                let binding = tagOpBinding(for: group.uuid)
                filterSection(group.name.capitalized, op: binding, ops: [.allOf, .exact]) {
                    FlowLayout(spacing: 6) {
                        ForEach(group.tags) { tag in
                            let isSelected = selectedTagsByGroup[group.uuid]?.contains(tag.uuid) ?? false
                            FilterChip(
                                label: tag.name.replacing("-", with: " ").capitalized,
                                isSelected: isSelected,
                                isDisabled: !isSelected && !availableTagUUIDs.contains(tag.uuid)
                            ) {
                                withAnimation(.snappy) {
                                    var set = selectedTagsByGroup[group.uuid] ?? []
                                    if set.contains(tag.uuid) {
                                        set.remove(tag.uuid)
                                    } else {
                                        set.insert(tag.uuid)
                                    }
                                    selectedTagsByGroup[group.uuid] = set
                                }
                            }
                        }
                    }
                }
            }
        }
        .padding(.horizontal, 20)
        .padding(.top, 8)
        .padding(.bottom, 16)
    }

    private func filterSection<Content: View>(
        _ title: String, @ViewBuilder content: () -> Content
    ) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(title)
                .font(.caption.weight(.semibold))
                .foregroundStyle(.secondary)
                .textCase(.uppercase)
                .tracking(0.5)
            content()
        }
    }

    private func filterSection<Content: View>(
        _ title: String,
        op: Binding<FilterOp>,
        ops: [FilterOp],
        @ViewBuilder content: () -> Content
    ) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack(alignment: .firstTextBaseline) {
                Text(title)
                    .font(.caption.weight(.semibold))
                    .foregroundStyle(.secondary)
                    .textCase(.uppercase)
                    .tracking(0.5)

                Spacer()

                opPicker(selection: op, options: ops)
            }
            content()
        }
    }

    private func opPicker(selection: Binding<FilterOp>, options: [FilterOp]) -> some View {
        HStack(spacing: 0) {
            ForEach(options, id: \.self) { op in
                let isActive = selection.wrappedValue == op
                Button {
                    withAnimation(.snappy(duration: 0.2)) {
                        selection.wrappedValue = op
                    }
                } label: {
                    Text(op.displayName)
                        .font(.caption2.weight(.medium))
                        .padding(.horizontal, 8)
                        .padding(.vertical, 3)
                        .background(isActive ? Color.accentColor.opacity(0.15) : .clear)
                        .foregroundStyle(isActive ? Color.accentColor : .secondary)
                }
                .buttonStyle(.plain)
            }
        }
        .background(Color(.systemGray6))
        .clipShape(.capsule)
    }

    private func tagOpBinding(for groupUUID: String) -> Binding<FilterOp> {
        Binding(
            get: { tagOpByGroup[groupUUID] ?? .allOf },
            set: { tagOpByGroup[groupUUID] = $0 }
        )
    }

    private func gridImageURL(_ uuid: String) -> URL {
        useThumbnails ? api.thumbnailURL(uuid: uuid) : api.imageURL(uuid: uuid)
    }

    // MARK: - Data Loading

    private func loadFilterData() async {
        async let c = api.fetchCollections()
        async let m = api.fetchModels()
        async let t = api.fetchTags()
        do {
            collections = try await c
            allModels = try await m
            tagGroups = try await t
        } catch {
            // error handling
        }
        isLoadingData = false
        await performSearch()
    }

    // MARK: - Search

    private var currentFilters: [FilterClause] {
        var filters: [FilterClause] = []
        if let col = selectedCollection {
            filters.append(FilterClause(field: .collection, op: .eq, value: .single(col)))
        }
        if !selectedModelUUIDs.isEmpty {
            filters.append(
                FilterClause(
                    field: .models, op: modelOp, value: .multiple(Array(selectedModelUUIDs)))
            )
        }
        for group in tagGroups {
            let selected = selectedTagsByGroup[group.uuid] ?? []
            if !selected.isEmpty {
                let op = tagOpByGroup[group.uuid] ?? .allOf
                filters.append(
                    FilterClause(field: .tags, op: op, value: .multiple(Array(selected)))
                )
            }
        }
        return filters
    }

    private func debouncedSearch() {
        searchTask?.cancel()
        searchTask = Task {
            try? await Task.sleep(for: .milliseconds(300))
            guard !Task.isCancelled else { return }
            await performSearch()
        }
    }

    private func performSearch() async {
        isSearching = true
        do {
            filterOptions = try await api.searchFilterOptions(filters: currentFilters)
        } catch {
            if !Task.isCancelled {
                filterOptions = nil
            }
        }
        isSearching = false
    }

    private func fetchImages() async {
        isLoadingImages = true
        do {
            images = try await api.searchImages(filters: currentFilters)
        } catch {
            if !Task.isCancelled {
                images = []
            }
        }
        isLoadingImages = false
    }
}

extension FilterOp {
    var displayName: String {
        switch self {
        case .anyOf: "Any"
        case .allOf: "All"
        case .exact: "Exact"
        case .eq: "Eq"
        case .noneOf: "None"
        }
    }
}

extension Int: @retroactive Identifiable {
    public var id: Int { self }
}

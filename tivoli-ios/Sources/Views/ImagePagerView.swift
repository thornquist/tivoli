import SwiftUI

struct ImagePagerView: View {
    @Binding var images: [ImageSummary]
    let initialIndex: Int

    @Environment(APIClient.self) private var api
    @Environment(\.dismiss) private var dismiss
    @State private var currentIndex: Int
    @State private var showTagEditor = false
    @State private var dragOffset: CGFloat = 0

    init(images: Binding<[ImageSummary]>, initialIndex: Int) {
        self._images = images
        self.initialIndex = initialIndex
        self._currentIndex = State(initialValue: initialIndex)
    }

    var body: some View {
        ZStack {
            Color.black.ignoresSafeArea()

            TabView(selection: $currentIndex) {
                ForEach(Array(images.enumerated()), id: \.element.id) { index, image in
                    CachedAsyncImage(url: api.imageURL(uuid: image.uuid))
                        .aspectRatio(contentMode: .fit)
                        .frame(maxWidth: .infinity, maxHeight: .infinity)
                        .tag(index)
                        .onTapGesture {
                            withAnimation { showTagEditor.toggle() }
                        }
                }
            }
            .tabViewStyle(.page(indexDisplayMode: .automatic))
            .ignoresSafeArea()
            .offset(y: dragOffset)
            .gesture(
                DragGesture(minimumDistance: 30, coordinateSpace: .global)
                    .onChanged { value in
                        if value.translation.height > 0 {
                            dragOffset = value.translation.height
                        }
                    }
                    .onEnded { value in
                        if value.translation.height > 100 {
                            dismiss()
                        } else {
                            withAnimation(.snappy(duration: 0.25)) {
                                dragOffset = 0
                            }
                        }
                    }
            )

            if showTagEditor, currentIndex < images.count {
                VStack {
                    Spacer()
                    TagEditorView(imageUUID: images[currentIndex].uuid)
                        .transition(.move(edge: .bottom))
                }
            }
        }
        .background(Color.black.opacity(1 - Double(dragOffset) / 400).ignoresSafeArea())
        .statusBarHidden()
    }
}
